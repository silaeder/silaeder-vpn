-- Create table for session information

CREATE TABLE Sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid VARCHAR NOT NULL,
    token VARCHAR NOT NULL
);

-- Create Users table
CREATE TABLE Users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    custom_id VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    hashed_password VARCHAR NOT NULL,
    permission INTEGER NOT NULL,
    uuid VARCHAR NOT NULL,
    session INTEGER
);

-- Add Admin account
INSERT INTO Users
VALUES (
    NULL,
    "Dmitri Proskuriakov",
    "JustDprroz",
    "dmitri.proskuriakov@gmail.com",
    "$pbkdf2-sha256$i=10000,l=32$pp8JWbAvLiDGkxp9iQbzww$a33xwXw6INY+J87XaJP7At07lMlMh5rh3xHQI8R6LrU",
    "10",
    "844a3466-85d1-4674-818e-f3d3764378de",
    NULL
);

-- Create table for servers
CREATE TABLE Servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key VARCHAR NOT NULL,
    private_key VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    info VARCHAR NOT NULL
);

-- Create Peers table
CREATE TABLE Peers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key VARCHAR NOT NULL,
    private_key VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    server_public_key VARCHAR NOT NULL,
    server_address VARCHAR NOT NULL,
    owner_uuid VARCHAR NOT NULL,
    owner_name VARCHAR NOT NULL
);