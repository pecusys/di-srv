CREATE TABLE IF NOT EXISTS FieldEntries( 
    id SERIAL PRIMARY KEY NOT NULL,
    eeid INTEGER NOT NULL REFERENCES EntryTypes(id),
    fid INTEGER NOT NULL REFERENCES Fields(id),
    content TEXT
);
