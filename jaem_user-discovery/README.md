# UserDiscoveryService-Rust-
Rust implementation of User Discovery Service
```

## API Documentation

### Overview
This API provides a simple user management service over a TCP connection. It listens on `127.0.0.1:8080` and processes user-related requests.

### Endpoints

#### 1. `GET /users`
**Description:** Retrieves a list of all users stored in the system.

**Request Format:**
```http
GET /users HTTP/1.1
```

**Response Format:**
```json
{
    "users": [
        {
            "id": 1,
            "name": "John Doe",
            "email": "john.doe@example.com"
        }
    ]
}
```

#### 2. `POST /users`
**Description:** Adds a new user to the system.

**Request Format:**
```http
POST /users HTTP/1.1
Content-Type: application/json

{
    "name": "Jane Doe",
    "email": "jane.doe@example.com"
}
```

**Response Format:**
```json
{
    "message": "User added successfully",
    "user": {
        "id": 2,
        "name": "Jane Doe",
        "email": "jane.doe@example.com"
    }
}
```

#### 3. `DELETE /users/{id}`
**Description:** Deletes a user by ID.

**Request Format:**
```http
DELETE /users/1 HTTP/1.1
```

**Response Format:**
```json
{
    "message": "User deleted successfully"
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


