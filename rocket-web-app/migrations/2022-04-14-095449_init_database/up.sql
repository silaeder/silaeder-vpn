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
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    hashed_password VARCHAR NOT NULL,
    permission INTEGER NOT NULL,
    uuid VARCHAR NOT NULL
);

-- Add Admin account
INSERT INTO Users
VALUES (
    NULL,
    "administrator",
    "admin",
    "admin@silaeder.vpn",
    -- Hash of "adminpassword"
    "$pbkdf2-sha256$i=10000,l=32$FMlgbA/mXH5IMhlWwywsNw$8ZYSqomcW9FmU+lftD0fLKCwPIZN3uJSHLvo+i4adek",
    "10",
    "b26207db-bcf3-4442-ae79-41ec3e5cc9b5"
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