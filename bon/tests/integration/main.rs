use bon::{bon, builder};

#[test]
fn smoke() {
    #[builder]
    fn sut(
        /// ### Documentation
        /// **Docs** for arg1.
        ///
        /// Multiline with `code` *examples* __even__!
        ///
        /// ```
        /// let wow_such_code = true;
        /// println!("Code is so lovely! {wow_such_code}");
        /// ```
        ///
        /// - List item 1
        /// - List item 2
        arg1: bool,

        /// Docs for arg2
        arg2: &'_ str,
        arg3: String,
        arg4: u32,
        arg5: Option<u32>,
        arg6: Option<&str>,
        arg7: Vec<String>,
        arg8: (u32, &[bool]),
    ) -> String {
        let _ = (arg1, arg2, arg4, arg5, arg6, arg7, arg8);
        arg3
    }

    let actual = sut()
        .arg1(true)
        .arg2("arg2")
        .arg3("arg3".to_owned())
        .arg4(1)
        .arg7(vec!["arg7".to_owned()])
        .arg8((1, &[true]))
        .build();

    assert_eq!(actual, "arg3");
}

#[test]
fn lifetime_elision() {
    #[builder]
    fn sut(arg: &str, _arg2: usize) -> (&str, &str, [&str; 1]) {
        (arg, arg, [arg])
    }

    let actual = sut().arg("blackjack")._arg2(32).build();
    assert_eq!(actual, ("blackjack", "blackjack", ["blackjack"]));
}

#[tokio::test]
async fn async_func() {
    #[builder]
    async fn sut<Fut: std::future::Future>(fut: Fut) -> Fut::Output {
        fut.await
    }

    let actual = sut().fut(async { 42 }).build().await;
    assert_eq!(actual, 42);
}

#[test]
fn unsafe_func() {
    #[builder]
    unsafe fn sut(arg: bool) {
        let _ = arg;
    }

    let builder = sut().arg(true);

    // Only the call method should be unsafe
    unsafe { builder.build() };
}

#[test]
fn impl_traits() {
    #[builder]
    fn sut(
        /// Some documentation
        iterable: impl IntoIterator<Item = impl Into<u32>>,
        showable: impl std::fmt::Display + std::fmt::Debug,
    ) -> (String, Vec<u32>) {
        let str = format!("{showable} + {showable:#?}");
        let vec = iterable.into_iter().map(Into::into).collect();

        (str, vec)
    }

    let (str, vec) = sut()
        .iterable(vec![1_u32, 2, 3])
        .showable("showable")
        .build();

    assert_eq!(str, "showable + \"showable\"");
    assert_eq!(vec, [1, 2, 3]);
}

#[test]
fn constructor() {
    struct Counter {
        val: u32,
    }

    #[bon]
    impl Counter {
        #[builder]
        fn builder(initial: Option<u32>) -> Self {
            Self {
                val: initial.unwrap_or_default(),
            }
        }
    }

    let counter = Counter::builder().initial(Some(3)).build();

    assert_eq!(counter.val, 3);
}

#[test]
fn receiver() {
    #[derive(Clone)]
    struct Counter {
        val: u32,
    }

    #[bon]
    impl Counter {
        #[builder]
        fn increment(&self, disabled: bool) -> Self {
            if disabled {
                return self.clone();
            }
            Self { val: self.val + 1 }
        }
    }

    let counter = Counter { val: 0 };
    let counter = counter.increment().disabled(false).build();

    assert_eq!(counter.val, 1);
}

#[test]
fn receiver_with_lifetimes() {
    struct Sut<'a, 'b> {
        a: &'a str,
        b: &'b str,
    }

    #[bon]
    impl Sut<'_, '_> {
        #[builder]
        fn method(&self, c: &str) -> String {
            let Self { a, b } = self;

            format!("{a}{b}{c}")
        }
    }

    let actual = Sut { a: "a", b: "b" }.method().c("c").build();
    assert_eq!(actual, "abc");
}

#[test]
fn self_in_a_bunch_of_places() {
    struct Sut;

    #[bon]
    impl Sut
    where
        Self: Sized + 'static,
    {
        #[builder]
        fn method(&self, me: Option<Self>) -> impl Iterator<Item = Self>
        where
            Self: Sized,
        {
            me.into_iter()
        }
    }

    assert_eq!(Sut.method().me(Some(Sut)).build().count(), 1);
}

#[test]
fn receiver_is_non_default() {
    struct Sut {
        str: String,
    }

    #[bon]
    impl Sut {
        #[builder]
        fn method(self: &Sut) -> &str {
            &self.str
        }
    }

    let sut = Sut {
        str: "blackjack".to_owned(),
    };

    assert_eq!(sut.method().build(), "blackjack");
}

#[test]
fn impl_block_ty_contains_a_reference() {
    struct Sut<T>(T);

    #[bon]
    impl<T> Sut<&'_ T> {
        #[builder]
        fn get(&self) -> &T {
            self.0
        }
    }

    assert_eq!(Sut(&42).get().build(), &42);
}

#[test]
fn impl_block_with_self_in_const_generics() {
    #[derive(Default)]
    struct Sut<const N: usize>;

    impl<const N: usize> Sut<N> {
        const fn val(&self) -> usize {
            42
        }
    }

    #[bon]
    impl Sut<{ Sut::<3>.val() }>
    where
        Self:,
    {
        #[builder]
        fn method(self) -> usize {
            self.val()
        }
    }

    assert_eq!(Sut::<42>.method().build(), 42);
}

#[test]
fn generics_with_lifetimes() {
    #[builder]
    fn sut<T>(arg: &&&&&T) {
        let _ = arg;
    }

    sut().arg(&&&&&&&&&&42).build();
}
