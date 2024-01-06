# `.tree` File Format

The `.tree` file format allows you to store and query tree structures efficiently. It's not a human-readable format, as its data is stored in bytes and your program interprets it as its own data.

## Glossary

| Term      | Definition                                                                                                                                                       |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tree      | A tree is a collection of items connected by branches following a hierarchy.                                                                                     |
| Item      | An item is a collection of sub-items. It has exactly one parent, zero or more children, and at least one sub-item.                                               |
| Sub-item  | A piece of data that belongs to an item.                                                                                                                         |
| Branch    | The connection between a parent item and its child items.                                                                                                        |
| Level     | The amount of items that has on top of it (parents). Level 0 means the root of the tree. Level 2 means that the item has a parent whose parent is the root item. |
| Root item | The top item of the tree (where the tree starts). It has no parent.                                                                                              |

## Headers

Headers specify how the data is stored inside the file. Correctly configuring the file headers is important to optimize performance (and file size).

### Identification

> [0; 8)

The file starts with the following 8 bytes: `4e 45 4b 4f 54 52 45 45`. Those must be the first 8 bytes of the file.

### Format Version

> [8; 10)

The next two bytes are the version of the format represented in binary.

#### Versions

| Version | Bytes   |
| ------- | ------- |
| 1       | `00 00` |

### Features

> [10; 12)

The next two bytes represent the features enabled in the tree. A `1` means that the feature is enabled. Bits that don't represent a feature can be ignored. Extra bits mean the amount of bits that will be added to each item if the feature is enabled.

| Bit | Feature    | Description                                    | Extra bits |
| --- | ---------- | ---------------------------------------------- | ---------- |
| 0   | Disabling  | Allows to disable a branch's and it's children | 1          |

> [!IMPORTANT]
> The order of the features by the bit that toggles them is important later when adding data to each tree item.

### Sub-items

> [12; -)

This header specifies the number and size of the sub-items in the tree. The order of the sub-items is kept.

```
[4 bytes: Amount of sub-items]
(
    [4 bytes: Sub-item size]
    for subitem in 0..amount_of_subitems
)
```

#### Amount of Sub-items

> [12; 16)

The amount of sub-items each tree item contains, represented in binary.

#### Sub-item Size

> [16; -)

The size of each sub-item in bits, represented in binary. Each item size takes four bytes, and none of the sub-item sizes can be missing.

## Tree

The tree can store anything that can be represented in bits. Each program can read the file and interpret it as its own data.

Trees are stored by flattening the tree in the following way:

```
     A        <- Level 0
   /   \
  B     C     <- Level 1
 / \   / \
D   E F   G   <- Level 2
```

Is converted to:

```
0 1  2
A BC DEFG
```

That way, the tree is flattened. The tree requires exactly 2 branches per item. The last level won't have any branches.

### Tree Items

Each item in the tree consists of one or more sub-items (defined in the headers). Each sub-item has a fixed length in bits (defined in the headers) and must follow that size exactly.

#### Features

These features can be enabled and disabled in the headers. Check the features headers above for more information.

Each item has its own headers specified by the features above. The order of the headers on top is kept.  Header bits must be present even if they aren't needed for the specific item.

Headers must be present even if they don't mean anything for that specific item. E.g. root items cannot be flattened, but they'll still have a bit assigned to flattening, which will be ignored.

The following graph represents how items must be structured:

```
(
    [n bit: Header]
    for n in item_header_sizes
)
(
    [n bits: Sub-item 1 content]
    for n in subitem_sizes
)
```

You must not add bits corresponding to features that aren't enabled. The amount of bits assigned to each feature can be checked [in the features header](#features).

##### Disabling

Disabling allows you to disable branches and their children. Note that they still occupy the same amount of bits than the other items, but they won't be taken into account when reading the tree. Enabling this feature allows you to store asymmetric trees. For example:

```
     A
   /   \
  B     C
 / \   / \
x   x F   G
```

In the example above, B still has its own items, but they're disabled so they're ignored when processing the tree. This is to keep the tree's structure predictable for more efficient querying.

###### How to Disable Items

All items must have an extra 1-bit prefix when this feature is enabled. This bit enables (0) or disables (1) the item. If an item is disabled, the item's content bits can be ignored. Note that they still MUST be present.

#### Sub-items

Each item's sub-item is a piece of data stored in that specific item. They don't have individual headers and are placed one after the other.

Sub-items are placed one after the other, in order of definition in the file headers.

## File Structure Graph

The following "graph" illustrates how a complete `.tree` file looks:

```
{ Headers:
    [8 bytes: File identifier]
    [2 bytes: Format version]
    [2 bytes: Features]
    [4 bytes: Amount of items]
    (
        [4 bytes: Item x size] 
        for x in amount_of_items
    )
}
{ Tree:
    (
        [n bits: Item header]
        for n in item_header_sizes
    )
    (
        [n bits: Sub-item content]
        for n in subitem_sizes
    )
}
```