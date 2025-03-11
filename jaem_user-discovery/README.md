# UserDiscoveryService-Rust-

## API Documentation

### Overview
This API provides a simple user management service over a TCP connection. It listens on `0.0.0.0:3000` and processes user-related requests.

### Endpoints

#### 1. `GET /users/{page}/{page_size}`
**Description:** Retrieves multiple users.

**Request Format:**
```http
GET /users/1/10 HTTP/1.1
```

**Response Format:**
```json
[
    {
        "uid": "123",
        "username": "John Doe",
        "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...],
        "profile_picture": "default.png"
        "description": "Hello World"
    },
    ...
]
```


#### 2. `GET /search_users/{username}/{page}/{page_size}`
**Description:** Retrieves multiple users by a pattern in their name.

**Request Format:**
```http
GET /users/John%20Doe HTTP/1.1
```

**Response Format:**
```json
[
    {
        "uid": "123",
        "username": "John Doe",
        "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...],
        "profile_picture": "default.png"
        "description": "Hello World"
    },
    ...
]
```

#### 3. `GET /user_by_uid/{uid}`
**Description:** Retrieves one user exactly by uid.

**Request Format:**
```http
GET /user_by_uid/123 HTTP/1.1
```

**Response Format:**
```json
{
    "uid": "123",
    "username": "John Doe"
    "public_keys": [{"algorithm":"ED25519","key":"Your Public Key"}, ...]
    "profile_picture": "aowiudgo18724612ougd1o38f710387"
    "description": "Hello World"
}

```

#### 4. `POST /create_user`
**Description:** Creates a new User

**Request Format:**
```http
POST /add_pub_key HTTP/1.1
Content-Type: application/json

{
    "uid:" "1234",
    "username": "John Doe",
    "public_keys": [{"algorithm":"ED25519","signature_key":"mySignatureKey", "exchange_key":"Hello_World", "rsa_key": "Not so secret"}, ...],
    "profile_picture": "123123123"
    "description": "Hello World"
}
```

**Response Format:**
```http
message: 'User added'
```

#### 5. `POST /add_pub_key`
**Description:** Adds a public key to an existing user. 

**Request Format:**
```http
POST /add_pub_key HTTP/1.1
Content-Type: application/json

{
    "uid:" "1234",
    "public_keys": [{"algorithm":"ED25519","signature_key":"my ed", "exchange_key":"jaek", "rsa_key": "Very secret"}, ...],
}
```

**Response Format:**
```http
    message: 'Public keys added',
```

#### 6. `DELETE /user/{uid}/{signature_key}`
**Description:** Deletes a public key from a user.

**Request Format:**
```http
DELETE /delete_pub_key/John%20Doe/Your%20Public%20Key HTTP/1.1
```

**Response Format:**
```http
    message: "Public key deleted"
```

### Error Handling
All error responses follow this format:
```json
{
    "error": "Description of the error"
}
```

### 7. `DELETE /user/{uid}`
**Description:** Deletes a user.

**Request Format:**
```http
DELETE /delete_user/John%20Doe HTTP/1.1
```

**Response Format:**
```http
    message: "User deleted"
```

### 8. `PATCH /profile`
**Description:** Updates a user's profile picture, description or username.

**Request Format:**
```http
PATCH /profile HTTP/1.1
Content-Type: application/json

{
    "uid": "123",
    "username": "John Doe",
    "profile_picture": "123123123",
    "description": "Hello World"
}
```

**Response Format:**
```http
    message: "Profile updated"
```

### Notes
- The API communicates over a raw TCP connection.
- Requests and responses follow HTTP-like formatting.
- User data is stored in a JSON file (`users.json`).


