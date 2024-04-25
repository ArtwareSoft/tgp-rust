

component('runCtx', {
    params: [
        {ctx, RTType: 'Ctx'}
    ],
    impl: pipe(
        Var(profile, ctx.profile),
        Switch(
            data(typeOf(profile)),
            Case('string', calcExpression(profile, ctx)),
            Case('boolean', profile),
            Case('number', profile),
            Case(profile.$asIs, profile.$asIs),
            Case('function', profile(ctx.cmpCtx)),
            Case('profile', pipe(
                Var(profileAsFunc, pipe(
                    Var(ctx, newCtx(ctx,{vars: profile.vars, data: profile.data})),
                    Var(pt, profile.pt),
                    Var(comp, propVal(ctx.comps,pt)),
                    Var(impl, { path: '%$pt%/impl', profile: comp.impl }),
                    Var(compArgs, mapProps(comp.params, pipe(
                        Var(parentParam),
                        Var(propId, parentParam.id),
                        propVal(profile, propId),
                        If(
                            isArray(profile), 
                            pipe(profile, runCtx(newCtx(ctx,{addToPath: '%$propId%/%$index%', profile, parentParam})), {index: true})
                            runCtx(newCtx(ctx,{addToPath: propId, profile, parentParam }))
                        )
                    )), {type: 'RTArgs'}),
                    runCtx(newCtx(ctx,{cmpCtx: ctx, compArgs, resetPath: impl.path, profile: impl.profile}))
                , { type: 'function', signiture: 'ctx<%$comp/RTInput%> => %$ctx/parentParam/as%'}),
                If(ctx.parentParam.dynamic, profileAsFunc, profileAsFunc(ctx))
            )),
          ),
        )
        castToType(ctx.parentParam.as)
    )
})

