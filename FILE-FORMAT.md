# `.tree` File Format

The `.tree` file format allows you to store and query tree structures efficiently. It's not a human-readable format, as its data is stored in bytes and your program interprets it as its own data.

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
| 1       | `00 01` |

### Features

> [10; 12)

The next two bytes represent the features enabled in the tree. A `1` means that the feature is enabled. Bits that don't represent a feature can be ignored.

| Bit | Feature    | Description                                  |
| --- | ---------- | -------------------------------------------- |
| 0   | Flattening | Allows setting more than one item per branch |

> [!IMPORTANT]
> The order of the features by the bit that toggles them is important later when adding data to each tree item.

### Items

> [12; -)

This header specifies the number and size of the items in the tree. The order of the items is kept.

#### Amount of Items

> [12; 16)

The amount of items each tree item contains, represented in binary.

#### Item Size

> [16; -)

The size of each tree item, represented in binary. Each item size takes four bytes, and none of the item sizes can be missing.

```
[4 bytes: Amount of items] [4 bytes: Item 1 size] [4 bytes: Item 2 size] ...
```

## Tree

The tree can store anything that can be represented in bits. Each program can read the file and interpret it as its own data.