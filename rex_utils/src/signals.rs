//! This module provides a simple API over signals implemented with boxed closures.
//!
//! To simplify the API, the implementation is very heavily implemented by macros. As such, the
//! documentation for the API is a bit hidden by the types creates by the macros. The signal allows
//! connecting a single closure that can be signaled multiple times.
//!
//! # Usage
//!
//! This module is all macros, to use it you should declare the create in your main.rs or lib.rs
//! as:
//!
//! ```ignore
//! #[macro_use] extern crate rex_utils;
//! ```
//!
//! # Declaring a signal
//!
//! The first part of the signal API is declaring the signal with the type of the arguments. This
//! is done with the ```signal_decl!``` macro, that accepts a type name and argument types. The
//! created type has three methods. The ```connect(&mut self, f: Box<FnMut(...)>)``` we will
//! discuss later with the ```signal!``` macro. The ```signal(&mut self, ...)``` is used by the owner
//! of the signal to trigger the connected closure. Lastly, a ```new()``` method for creating an
//! instance of signal (the ```Default``` trait can also be used). For example, a "text change"
//! event commonly seen in GUI toolkits would look like this:
//!
//! ```ignore
//! // This is how we will declare the type of the signal. This signal passes an owned string
//! // and an isize.
//! signal_decl!{TextChangedEvent(String, i32)}
//!
//! // Now, let's create a fake textbox that will have this signal. Pretend that we have implemented
//! // the rest.
//! struct TextBox {
//!     on_text_changed: TextChangedEvent,
//! }
//!
//! let tb = TextBox {
//!     on_text_changed: TextChangedEvent::new(),
//!     // Also the default trait can be used:
//!     // on_text_changed: Default::default()
//! };
//!
//! // Let's ignore for now how to connect a closure to a signal.
//! // Now let's signal the closure.
//! tb.on_text_changed.signal("Please enter text".to_string(), 0);
//! ```
//!
//! # Receiving and dispatching signaled signals
//!
//! The second part of the signal API is declaring the signal receiver. The signal receiver can
//! be thought of as the object that the triggered signal is sent to. This helps decouple the
//! closure that is connected to the signal from needing to have a reference to the object the
//! closure will use. This also allows the closure to have a mutable reference and sidestep many
//! lifetime issues with closures used in signals and moves the complexity to the implementation.
//!
//! A signal receiver is declared with the ```signalreceiver_decl!``` macro, that accepts the type
//! of the object that the signal will be "posted" to. The created type has two methods. The first,
//! ```run(&self, &mut ObjType)``` to dispatch any incoming signals. Additionally a
//! ```new()``` method to create a type (though the Default trait can also be used).
//! For example, here is how we would create a signal receiver for our App struct:
//!
//! ```ignore
//! struct App {
//!     bytes_changed: i32
//! }
//!
//! signalreceiver_decl!{AppSignalReceiver(App)}
//!
//!
//! // Say we have an instance of our App
//! let app = App { bytes_changed: 0 };
//!
//! // Now let's create an instace of the signal receiver
//! let sr = AppSignalReceiver::new();
//! // Additionally, the Default trait can be used
//! // let sr: AppSignalReceiver = Default::default();
//!
//! // We will ignore for now how signals are connected and posted.
//! // Once a signal is posted, to run the closure that was connected to our object we will use
//! // the run method.
//! sr.run(app)
//! ```
//!
//! # Connecting a closure to a signal on a specific receiver
//!
//! So now we just need to put everything together. For that we will use the last macro,
//! ```signal!(signal_receiver with |obj, type..| <closure body>)```. Though the macro seems to
//! accept a closure the macro really splits it up into 2 seperate closures to allow the decoupling
//! from the signal and the signal receiver. This macro will create the boxed closure that can be
//! passed to the signal's ```connect``` method. Let's continue with the (contrived) example:
//!
//! ```ignore
//! tb.connect(
//!     signal!(sr with |obj, str_changed, index_changed| obj.bytes_changed = str_changed.len())
//! );
//! ```
//!
//! Note: Due to the closure arguments not being proper closure arguments, there is a bit of magic
//! there. Specifically, the type of obj is ```&mut Type```, where Type is the type supplied to
//! the signal_receiver.
//!
//! # Complete Example
//!
//! Here is a complete example based on what we had til now:
//!
//! ```ignore
//! // An event signaled when text is changed in a text box
//! signal_decl!{TextChangedEvent(String, i32)}
//!
//! // A fake text box
//! struct TextBox {
//!     on_text_changed: TextChangedEvent,
//! }
//!
//! // A fake app interested in changed to a text box
//! struct App {
//!     bytes_changed: i32
//! }
//!
//! // The signal receiver for our fake app
//! signalreceiver_decl!{AppSignalReceiver(App)}
//!
//!
//! // Let's create all of our objects
//! let tb = TextBox {
//!     on_text_changed: TextChangedEvent::new(),
//! };
//!
//! let app = App { bytes_changed: 0 };
//!
//! let sr = AppSignalReceiver::new();
//!
//!
//! // Now let's connect a closure to the signal
//! tb.connect(
//!     signal!(sr with |obj, str_changed, index_changed| {
//!         println!("Closure is being run!");
//!         obj.bytes_changed = str_changed.len());
//! });
//!
//! // Now let's signal the closure.
//! tb.on_text_changed.signal("Some text".to_string(), 0);
//!
//! // For now the closure has not run yet!
//! assert_eq!(app.bytes_changed, 0);
//!
//! // Once we let the signal receiver run all the queued closure will be run
//! sr.run(app)
//!
//! assert_eq!(app.bytes_changed, 9);
//! ```

#[macro_use]

/// Internal macro used by the signal module, should not be used.
#[macro_export]
macro_rules! ident_zip_signal {
    ( () ; ( $($id: ident,)* ) ; ( $($idr:ident: $tyr:ty,)* ) ) => {
        pub fn signal( &mut self, $($idr : $tyr,)* ) {
            if let Some(ref mut f) = self.s {
                f($($idr),*);
            }
        }
    };
    ( ($t0:ty, $($ty:ty,)*) ; ($id0:ident, $($id: ident,)*) ; ($($idr:ident: $tyr:ty,)*) ) => {
        ident_zip_signal!{($($ty,)*) ; ($($id,)*) ; ( $($idr: $tyr,)* $id0: $t0, ) }
    }
}

/// A macro used to declare a signals.
///
/// See the documentation in the ```signals``` module for more information.
#[macro_export]
macro_rules! signal_decl {
    ( $name:ident($($t:ty ),*) ) => {

        pub struct $name {
            s: Option<Box<FnMut($($t),*)>>,
        }

        impl $name {
            #[allow(dead_code)]
            pub fn new() -> $name {
                Default::default()
            }

            ident_zip_signal!{($($t,)*) ; (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s,
                t, u, v, w, x, y, z,); ()}

            pub fn connect(&mut self, f: Box<FnMut($($t),*)>) {
                self.s = Some(f);
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name {
                    s: None,
                }
            }
        }

    }
}

/// A macro used for creating a closure to connect to a signal.
///
/// See the documentation in the ```signals``` module for more information.
#[macro_export]
macro_rules! signal {
    ( $sr:ident with |$obj:ident, $($id:ident),*| $bl:expr ) => ( {
        let sender_clone = $sr.sender.clone();
        Box::new(move |$($id),*| {
            sender_clone.send(Box::new(move |$obj|
                $bl
            )).unwrap();
        })
    })
}

/// A macro used for creating a signal receiver.
///
/// See the documentation in the ```signals``` module for more information.
#[macro_export]
macro_rules! signalreceiver_decl {
    ( $name: ident($t:ty) ) => {
        struct $name {
            receiver: ::std::sync::mpsc::Receiver<Box<FnMut(&mut $t)>>,
            sender: ::std::sync::mpsc::Sender<Box<FnMut(&mut $t)>>,
        }

        impl $name {
            fn new() -> $name {
                let (sender, receiver) = ::std::sync::mpsc::channel();
                $name {
                    sender: sender,
                    receiver: receiver,
                }
            }

            fn run(&self, ss: &mut $t) {
                loop {
                    match self.receiver.try_recv() {
                        Ok(mut handler) => handler(ss),
                        Err(_) => break,
                    }
                }
            }
        }
    }
}
