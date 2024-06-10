comp!(dataType, {
    type: TgpType,
    a: 5
});

comp!(plus, {
    type: Exp,
    params: [ 
        param(x, Exp), 
        param(y, Exp) 
    ],
    /*
    impl: (x: FuncType<Exp>, y: Exp) {
        x() + y
    },
    => impl2: |profile: &'static Profile| {
        match (profile.prop::<Exp>("x"), profile.prop::<Exp>("y")) {
            (x, y) => {
                x + y
            }
        }
    },
*/
    impl: |profile: &'static Profile| {
        profile.prop::<Exp>("x") + profile.prop::<Exp>("y")
    }
});