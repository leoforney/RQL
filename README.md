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
- Float = (f32)
- Text = (String)
- Boolean = (bool)

Example:

```
rql> CREATE TABLE people (id INTEGER NOT NULL UNIQUE,name TEXT NOT NULL,is_active BOOLEAN);    
Table 'people' created successfully.
```

#### `UPDATE`

Updates table values and their entirety using `rqle`. This expression language is very similar to WGSL compute shaders.

The operation runs on GPU or CPU using wgpu.

Example:

```
rql> UPDATE tempfloats SET tempcol = col1 * sin(2.0 * col2), col1 = 3.0 * col1, col2 = tempcol;
 col1       | col2 
------------+-------------
 2.96823    | 0.9789174 
 1.403913   | 0.23157208 
 1.111668   | 0.097824186 
 2.961729   | 0.37335587 
 2.297403   | 0.053336103 
 0.208929   | 0.049138047 
 2.678364   | 0.89191675 
 0.340038   | 0.09628068 
 1.7154751  | 0.5481875 
 0.131199   | 0.001619938 
 1.9013339  | 0.62547964 
 0.15069899 | 0.037533995 
 0.73967695 | 0.245279 
 1.452345   | 0.23686013 
 1.389264   | 0.08835022 
 1.396404   | 0.19898693 
 1.8931859  | 0.32209942 
 1.028994   | 0.2918854 
 0.510147   | 0.14591624 
 2.127111   | 0.70828396 
 1.206243   | 0.16141483 
 0.106755   | 0.006086269 
 2.274606   | 0.39827594 
 1.893918   | 0.54868615 
 2.951751   | 0.94490737 
 1.1834459  | 0.3919375 
```


