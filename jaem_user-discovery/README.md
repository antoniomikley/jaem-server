# UserDiscoveryService-Rust-

## API Documentation

### Overview
This API provides a simple user management service over a TCP connection. It listens on `127.0.0.1:8080` and processes user-related requests.

### Endpoints

#### 1. `GET /users/{name}`
**Description:** Retrieves multiple users by a pattern in their name.

**Request Format:**
```http
GET /users/John%20Doe HTTP/1.1
```

**Response Format:**
```json
[
    {
        "username": "John Doe"
        "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...]
    }
]
```

#### 2. `GET /user/{name}`
**Description:** Retrieves one user exactly by name.

**Request Format:**
```http
GET /user/John%20Doe HTTP/1.1
```

**Response Format:**
```json
[
    {
        "username": "John Doe"
        "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...]
    }
]
```

#### 3. `POST /add_pub_key`
**Description:** Adds a public key to a user. Adds a new user if the user does not exist. 

**Request Format:**
```http
POST /add_pub_key HTTP/1.1
Content-Type: application/json

{
    "username": "John Doe",
    "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...]
}
```

**Response Format:**
```json
{
    "message": "User added successfully",
    "user": {
        "username": "John Doe",
        "public_keys": [
            {"algorithm":"ED25519","key":"Your Public Key"}, 
                ...
        ]
    }
}
```

#### 4. `DELETE /user/{username}/{public_key}`
**Description:** Deletes a public key from a user.

**Request Format:**
```http
DELETE /delete_pub_key/John%20Doe/Your%20Public%20Key HTTP/1.1
```

**Response Format:**
```json
{
    "message": "Public key deleted successfully from John Doe"
}
```

#### 5. `DELETE /user/{username}`
**Description:** Deletes a user and all of its data.

**Request Format:**
```http
DELETE /user/John%20Doe HTTP/1.1
```

**Response Format:**
```json
{
    "message": "User John Doe deleted successfully"
}
```

### Error Handling
All error responses follow this format:
```json
{
    "error": "Description of the error"
}
```

### Notes
- The API communicates over a raw TCP connection.
- Requests and responses follow HTTP-like formatting.
- User data is stored in a JSON file (`users.json`).


