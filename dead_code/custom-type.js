comp(upperCase, {
    type: string,
})

comp(split, {
    type: array(string),
})

comp(obj, {
    type: object,
})

comp(sin, {
    type: number,
})

comp(data, { type: type })
comp(string, { type: type, impl: subType(data) })
comp(boolean, { type: type, impl: subType(data) })

comp(customType, {
    type: type,
    params: [
        param(baseType,type),
        param(refinedType,type)
    ],
    impl: 5,
})

comp(elemInPipe, {
    type: type,
    impl: customType(data, { 
        refinedType: typeOfProfile({inputType: If(prevSibling("%%"), typeOfProfile(prevSibling("%%")), '%$inputType%')}) 
    })
})

comp(pipe, {
    type: customType(data, { refinedType: typeOfProfile(firstSucceeding(last("%elems%"), "%source%")) }),
    params: [
        param(source, source),
        param(elems, array(elemInPipe(data)), {secondParamAsArray})
    ]
})

