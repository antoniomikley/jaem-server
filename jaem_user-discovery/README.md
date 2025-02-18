# UserDiscoveryService-Rust-
Rust implementation of User Discovery Service
```

## API Documentation

### Overview
This API provides a simple user management service over a TCP connection. It listens on `127.0.0.1:8080` and processes user-related requests.

### Endpoints

#### 1. `GET /search_by_name?username={name}`
**Description:** Retrieves the user details by name/id.

**Request Format:**
```http
GET /search_by_name?username=John%20Doe HTTP/1.1
```

**Response Format:**
```json
[
    {
        "id": "John Doe"
        "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...]
    }
]
```

#### 2. `POST /add_pub_key`
**Description:** Adds a public key to a user. Adds a new user if the user does not exist.

**Request Format:**
```http
POST /add_pub_key HTTP/1.1
Content-Type: application/json

{
    "id": "John Doe",
    "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...]
}
```

**Response Format:**
```json
{
    "message": "User added successfully",
    "user": {
        "id": "John Doe",
        "public_keys": [
            {"algorithm":"ED25519","key":"Your Public Key"}, 
                ...
        ]
    }
}
```

#### 3. `DELETE /delete_pub_key?username={name}&public_key={key}`
**Description:** Deletes a public key from a user.

**Request Format:**
```http
DELETE /delete_pub_key?username=John%20Doe&public_key=Your%20Public%20Key HTTP/1.1
```

**Response Format:**
```json
{
    "message": "Public key deleted successfully from John Doe"
}
```

#### 4. `DELETE /delete_user?username={name}`
**Description:** Deletes a user.

**Request Format:**
```http
DELETE /delete_user?username=John%20Doe HTTP/1.1
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


