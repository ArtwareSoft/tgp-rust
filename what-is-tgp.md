## What is TGP
TGP stands for *T*ype-*G*eneric Class-*P*rofile. 
TGP is a methodology & syntax for Domain Specific Languages (DSL) development.
TGP suggests *limitation* on DSLs to emplify some DSL qualities:
- Guided development experience where the software assists in the development process by presenting options along with their outcomes
- *Extendible* DSL
- DSL *integration*
- out of the box *tooling* - inteliscript, live preview, text completion, probe and circuit, work on result (WYSIWYG), vscode, web studio
- interpreter & built it "data glue lang"

TGP *does not* focus on:
- DSL performance
- Flexibility in DSL syntax
However using specific DSLs implemented via TGP, one can gain very effecient code and any syntax...

TGP lives inside other general purpose languages. It started in C++, extended to C# and java, and was rebuilt for javascript. Hopefully it will be soon implemented within RUST and python.
Yet quite unknown, TGP was implemented in large succesfull projects, mostly as the underlying technology for visual software products.

## TGP Model
The major limitation on DSLs enforced by TGP is the generic TGP language model.
Let's start with some examples:
### html/css
```
div(
    div(
        span("Hello", font(serif("Lucida Bright"), 14, {color: red}), background(rgb(15,15,15)) ) ,
        span("world", border(2), { class: "my-class", id: "myId" }),
    ),
    div({text: "hello world"}),
    table(
        tr(th("Month", width(100)), th("Saving")),
        tr(td("Jan"), td("$20")),
        tr(td("Feb"), td("$30"))
    )
)
```

Here is the html dsl schema definition

```
dsl(html)
comp(div, {
    type: elem,
    params: [
        param(elems, elem[]),
        param(text, string),
        param(style, property<css>[]),
        param(id, string),
        param(class, string),
        ...
    ]
})

comp(span, {
    type: elem,
    params: [
        param(text, string),
        param(style, property<css>[]),
        param(id, string),
        param(class, string),
        ...
    ]
})
...
```

and css schema definition
```
dsl(css)
comp(font, {
    type: property,
    params: [
        param(family, "font-family"),
        param(size, size),
        param(style, property<css>[]),
        param(id, string),
        param(class, string),
        ...
    ]
})

```
comp(serif, {
    type: "font-family",
    params: [
        param(specific, string[], { suggestions: ["Lucida Bright", "Lucida Fax", "Palatino", "Palatino Linotype", "Palladio", "URW Palladio"] }),
    ]
})

comp(monospace, {
    type: "font-family",
    params: [
        param(specific, string, { suggestions: ["Fira Mono", "DejaVu Sans Mono", "Menlo", "Consolas", "Liberation Mono", "Monaco", "Lucida Console", "monospace"] }),
        param(alternatives, "font-family[]")
    ]
})



```

```

```

### sql
select({
    from: employees,
    fields: [Name, Field("Home Address", {as : Address})],
    where: equal(Name,"David"),
    sortBy: Field("Department", {order: ascending})
})

