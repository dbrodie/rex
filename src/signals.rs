#[macro_use]

macro_rules! ident_zip_signal {
    ( () ; ( $($id: ident,)* ) ; ( $($idr:ident: $tyr:ty,)* ) ) => {
        fn signal( &mut self, $($idr : $tyr,)* ) {
            match self.s {
                Some(ref mut f) => f($($idr),*),
                None => ()
            }
        }
    };
    ( ($t0:ty, $($ty:ty,)*) ; ($id0:ident, $($id: ident,)*) ; ($($idr:ident: $tyr:ty,)*) ) => {
        ident_zip_signal!{($($ty,)*) ; ($($id,)*) ; ( $($idr: $tyr,)* $id0: $t0, ) }
    }
}

macro_rules! signal_decl {
    ( $name:ident($($t:ty ),*) ) => {

        struct $name {
            s: Option<Box<FnMut($($t),*)>>,
        }

        impl $name {
            fn new() -> $name {
                Default::default()
            }

            ident_zip_signal!{($($t,)*) ; (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z,); ()}

            fn connect(&mut self, f: Box<FnMut($($t),*)>) {
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

macro_rules! signal {
    ( $sr:ident with |$obj:ident, $($id:ident),*| $bl:expr ) => ( {
        let sender_clone = $sr.sender.clone();
        Box::new(move |$($id),*| {sender_clone.send(Box::new(move |$obj|
            $bl
        )).unwrap();})
    })
}

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

            fn run(&mut self, ss: &mut $t) {
                match self.receiver.try_recv() {
                    Ok(mut handler) => handler(ss),
                    Err(_) => (),
                }
            }
        }
    }
}
