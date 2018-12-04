# Cool deps!

## Understanding `dep-usage` output

### About Edges

Running `./pants clean-all dep-usage.jvm --no-cache-compile-zinc-read --no-summary --transitive testprojects/cooldeps/test_project/A` gives us the following output (with the annoying `//:scala-library-synthetic` removed):

```json
{
  "testprojects/cooldeps/test_project/A:A": {
    "cost": 73,
    "cost_transitive": 213,
    "dependencies": [
      {
        "aliases": [],
        "dependency_type": "unused",
        "products_used": 0,
        "products_used_ratio": 0.0,
        "target": "testprojects/cooldeps/test_project/B:B"
      },
      ...
    ],
    "products_total": 5
  },
  "testprojects/cooldeps/test_project/B:B": {
    "cost": 73,
    "cost_transitive": 140,
    "dependencies": [
      {
        "aliases": [],
        "dependency_type": "declared",
        "products_used": 1,
        "products_used_ratio": 0.14285714285714285,
        "target": "testprojects/cooldeps/test_project/C:C"
      },
      ...
    ],
    "products_total": 5
  },
  "testprojects/cooldeps/test_project/C:C": {
    "cost": 67,
    "cost_transitive": 67,
    "dependencies": [
      ...
    ],
    "products_total": 7
  }
}
```

The fun thing here is that, **even though A depends transitively on C**, the edge A->C doesn't appear, even with the `--transitive` flag on.

However, if we changed `A.scala` to be:
```scala
package A
import C1
case class A(name: String)
```

... then it would **still not show up**, since A doesn't actually use C at all, just imports it.

If `A.scala` were this however:

```scala
package A
import C1
case class A(name: String) {
   val c1 = C1(0f)
}
```

We **would** get the A->C edge, with this output:
```json
{
  "testprojects/cooldeps/test_project/A:A": {
    "cost": 73,
    "cost_transitive": 213,
    "dependencies": [
       ...
      {
        "aliases": [],
        "dependency_type": "undeclared",
        "products_used": 1, /* (a) */
        "products_used_ratio": 0.14285714285714285,
        "target": "testprojects/cooldeps/test_project/C:C"
      }
    ],
    "products_total": 5
  },
  ...
}
```
As you can see, in this case A uses one product of C, the constructor of the case class C1`(a)`.

This is cool, because we know that if we expect an edge to be there, and it's not, it's probably unused.


### About the stats:

This is what I think each field in the JSON means:

```json
{
  "testprojects/cooldeps/test_project/A:A": {
    "cost": 73, // Time to compile this target
    "cost_transitive": 213, // Time to compile this target and its (transitive) deps.
    "dependencies": [
       ...
      {
        "aliases": [],
        "dependency_type": "undeclared", // This means that C wasn't declaredin A's build file
        "products_used": 1, /* (a) */ // How many products (probaby symbol declarataions, we'll go over that) of C are used by A
        "products_used_ratio": 0.14285714285714285, // How many products of C are used by A, divided by the amount of products in C.
        "target": "testprojects/cooldeps/test_project/C:C"
      }
    ],
    "products_total": 5 // How many products A has.
  },
  ...
}
```

I think a `product` means (in the context of Scala) a Scala declaration, so `(case) class` declarations, `def` declarations and `val` and `var` declarations are all products.


