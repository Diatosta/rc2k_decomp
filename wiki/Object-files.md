# Object files

*Category: [Data formats](Data-formats)*

An object file holds one 3D shape of Rally Championship 2000: a tree, a fence, a spectator, a
car with all its parts, a piece of the menus' scenery. The files are named `OBJnnnn.RO3`,
where `nnnn` is the object's **kind**, a four-digit decimal number. The kind decides the
folder:

| Kind | Folder | What is there |
| --- | --- | --- |
| 0 .. `0x18F` (399) | `files/a/` | Scenery of the stages. |
| `0x190` .. `0x257` (400 .. 599) | `files/b/` | Cars and their parts. |
| `0x258` .. `0x31F` (600 .. 799) | `files/g/frnt/` | Objects of the front end (the menus). |
| `0x320` (800) and up | `files/s/extr/` | Extras. |

A stage says which kinds it uses in its `LOCALDAT` ([Course files](Course-files)); each
kind also has a record of settings in its folder's table
([Object and texture tables](Object-and-texture-tables)).

Beside most `.RO3` files lies an `.LWO` file of the same name (the model in LightWave's
object format), and one front-end object has an `.RO4` copy. The game never opens either:
only the `.RO3` is read.

All numbers are little-endian. "u32" is an unsigned 32-bit number, "i32" a signed one,
"f32" a 32-bit float. File names are matched without regard to case.

## Overall layout

| Part | Offset | Size |
| --- | --- | --- |
| Header | `0` | `0x2C` |
| Points | `0x2C` | point count × `0x0C` |
| Faces | after the points | varies: each face is `0x10` bytes plus `0x10` for each of its vertices |
| Normals | after the faces | normal count × `0x0C` |

The file ends with the last normal. A file may be at most `0x40000` (262144) bytes, and may
have at most `0x1000` (4096) each of points, faces and normals.

## Header

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u32 | Not read. 1 in the game's files. |
| `0x04` | u32 | Number of points. With 0 the object is empty. |
| `0x08` | u32 | Number of faces. With 0 the object is empty. |
| `0x0C` | u32 | Number of normals. May be 0: every normal reference is then taken as 0. |
| `0x10` | `0x1C` bytes | Not read. Zero in the game's files. |

## Points

Three i32 a point: x, y, z, whole numbers in the object's own space; y is height.

The cars' kinds (the `files/b/` range) are modelled lying along another axis. When the game
loads one it turns every point: the new x is the file's z, the new z is minus the file's x,
and a fixed ground clearance is taken off y.

## Faces

A face is a flat polygon with any number of corners; triangles and quadrilaterals are by far
the most common.

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u32 | Number of vertices, *n*. |
| `0x04` | u16 | The face's picture: the number of a texture picture, `TXnnnn.PCX` ([Textures (PCX)](Textures-PCX)). |
| `0x06` | u8 | Variant. 0: an ordinary face. Another value: the face belongs to an alternative look of a car's part and is drawn only while the car's state asks for that look (see `0x0B`). `0x1C` is a special variant tied to the wake a car part leaves; it is never drawn in the menus. |
| `0x07` | u8 | The low five bits are the face's **group**, 0 .. 31. |
| `0x08` | u8 | Flags. Bit 0 changes how the face is ordered among the others when drawn; bits 2 and 3 make the face fade with the object when it is far away. |
| `0x09` | 2 bytes | Not known. |
| `0x0B` | u8 | For a face with a variant: the value the car's state must have for it to show. |
| `0x0C` | u32 | The face's normal: a number in the normals list. |

Then *n* vertices of `0x10` bytes each:

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u32 | The vertex's point: a number in the points list. |
| `0x04` | i32 | Texture coordinate u, in 1/65536ths of a pixel of the picture: `0x00800000` is pixel 128. |
| `0x08` | i32 | Texture coordinate v, the same way. |
| `0x0C` | u32 | The vertex's normal (for smooth shading): a number in the normals list. |

### Groups

Groups split an object into parts that the game can show, hide or move on their own. For
scenery they hardly matter: the whole file is one object. For a car they are essential. One
car file holds the body and the parts that move or can come off, each in its own group; a
built-in table for each car model says which group number is which part. When a car is
loaded the game makes a separate object from each of eleven parts' groups, measures it, and
moves its points so that its middle is at 0: that is how a part can turn about its own
centre or fall off.

## Normals

Three f32 a normal: x, y, z, a direction of length 1.

## How the game loads one

The whole file is read into a work buffer. The game goes through the faces once to see which
are wanted (all of them, or those of one group), copies the points and normals those faces
use into its own tables with new numbers, and rewrites the faces to use the new numbers.
It also measures how far the object reaches from its upright axis, which the drawing uses to
decide whether the object is in view.

The game stops with an error if all objects together need more points or faces than it has
room for (it has room for `0xC000` (49152) faces in a stage).

## A made-up example

A square board standing upright, showing the whole of picture 123
(a 256 × 256 picture):

```
01 00 00 00  04 00 00 00  01 00 00 00  01 00 00 00    header: 1, 4 points, 1 face, 1 normal
00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00
00 00 00 00  00 00 00 00  00 00 00 00                 (header ends at 0x2C)

00 80 FF FF  00 00 00 00  00 00 00 00                 point 0: x -0x8000, y 0,       z 0
00 80 00 00  00 00 00 00  00 00 00 00                 point 1: x  0x8000, y 0,       z 0
00 80 00 00  00 00 01 00  00 00 00 00                 point 2: x  0x8000, y 0x10000, z 0
00 80 FF FF  00 00 01 00  00 00 00 00                 point 3: x -0x8000, y 0x10000, z 0

04 00 00 00  7B 00 00 00  00 00 00 00  00 00 00 00    face: 4 vertices, picture 123, group 0, normal 0
00 00 00 00  00 00 00 00  00 00 FF 00  00 00 00 00    vertex: point 0, u 0,   v 255
01 00 00 00  00 00 FF 00  00 00 FF 00  00 00 00 00    vertex: point 1, u 255, v 255
02 00 00 00  00 00 FF 00  00 00 00 00  00 00 00 00    vertex: point 2, u 255, v 0
03 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00    vertex: point 3, u 0,   v 0

00 00 00 00  00 00 00 00  00 00 80 BF                 normal 0: (0, 0, -1)
```

The file is `0x2C` + 4 × `0x0C` + `0x10` + 4 × `0x10` + `0x0C` = `0xB8` (184) bytes.

Code: `rc2k_core/src/scene/object_shape.rs` (the format and the choice by group),
`src/scene/load_object.rs` (loading, the cars' parts), `src/scene/set_object_file_names.rs`
(the names), `src/scene/list_object_face.rs` (variants, groups and flags when drawing).
