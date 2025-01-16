import sqlite3
import random
import math
import time

# Create a database connection and cursor
def create_database():
    conn = sqlite3.connect("test.db")
    cursor = conn.cursor()

    # Create the table
    cursor.execute("DROP TABLE IF EXISTS floats")
    cursor.execute("""
    CREATE TABLE floats (
        col1 REAL,
        col2 REAL,
        col3 REAL
    )
    """)
    conn.commit()
    return conn, cursor

# Populate the table with randomized values
def populate_table(cursor):
    rows = [(random.uniform(0, 1), random.uniform(0, 1), random.uniform(0, 1)) for _ in range(10_000_000)]
    cursor.executemany("INSERT INTO floats (col1, col2, col3) VALUES (?, ?, ?)", rows)

# Perform the calculation and write back to the database
def process_data(cursor):
    cursor.execute("SELECT rowid, col2, col3 FROM floats")
    rows = cursor.fetchall()

    updated_rows = []
    for row in rows:
        rowid, col2, col3 = row
        col1 = col2 * 3.0 * math.sin(5.0 * col3)
        updated_rows.append((col1, rowid))

    cursor.executemany("UPDATE floats SET col1 = ? WHERE rowid = ?", updated_rows)

# Main execution
if __name__ == "__main__":
    conn, cursor = create_database()
    print("Populating table with 10 million rows...")

    start_time = time.time()
    populate_table(cursor)
    conn.commit()
    print(f"Table populated in {time.time() - start_time:.2f} seconds.")

    print("Processing data...")
    start_time = time.time()
    process_data(cursor)
    conn.commit()
    end_time = time.time()

    print(f"Data processed and updated in {end_time - start_time:.2f} seconds.")

    conn.close()