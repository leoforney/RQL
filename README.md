# RQL

Structured database engine written in Rust, aimed to provide hardware accelerated statements. This library is
portable and can be run in browser or locally, as wgpu and compute shaders do all computations.

The method of storing is not the focus of the project at the current moment. It's current state is not designed to maximize
disk speed by using page tables and R* trees. The key goal is to accelerate complex mathematical queries.

When a table is created:
- Schema is initialized and saved in binary in schema/table_name_def.bin

When data is saved or updated:
- Data is stored in a single file per table in data/table_name_data.bin

There are no indexes at the moment

### Build

`cargo build`

### Usage

The operations that are currently supported:

#### `SELECT`

_Note: Only `SELECT *` allowed, order is not preserved_

Examples: 
```
rql> SELECT * FROM users WHERE is_active=false;
 is_active | name | id 
-----------+------+----
 false     | Ryan | 2 
 false     | Josh | 3 
```

```
rql> SELECT * FROM users;
 is_active | name | id 
-----------+------+----
 true      | Leo  | 1 
 false     | Ryan | 2 
 false     | Josh | 3 
 true      | Cole | 4 
```

#### `INSERT INTO`

_Note: doesn't require any "" for strings at the moment, just takes raw values until ,_

Example:

```
rql> INSERT INTO users VALUES (5, Gertrude, false) 
Row inserted successfully into table 'users'.

```

#### `CREATE TABLE`

Current schema values and their associated types:

- Integer = (i32)
- Float = (f64)
- Text = (String)
- Boolean = (bool)

Example:

```
rql> CREATE TABLE people (id INTEGER NOT NULL UNIQUE,name TEXT NOT NULL,is_active BOOLEAN);    
Table 'people' created successfully.
```


