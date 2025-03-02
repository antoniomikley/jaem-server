# Jaem Message Delivery

The Jaem Message Delivery Service provides the following endpoints:
- **/send_message**: to send a message
- **/get_messages**: to retrieve messages
- **/delete_message**: to delete messages

These endpoints are accessable via POST Method requests.

## /send_message

The /send_message enpoint can be used to send messages.
The message has to be contained in the request body in binary format.
Each Message has to start of with a single byte indicating the algorithm
used for signing messages, followed by the public key of the recipient.
Adhering to this format, it can be ensured that the recipient receives
the message, as long as the recipient can prove to be in possession of
the corresponding private key.

Currently ED25519 is the only supported signing algorithm which is 
indicated by a zero byte. A valid message would therefore be constructed in the
following manner:

`algorithm byte (1 Byte) + Public Key of the recipient (32 Bytes) + Message Content (up to 2^64 Bytes)`

## /get_messages

The `/get_messages` endpoint can be used to retrieve messages send to a specific
public key. A request to this enpoint has to contain a proof of authenticity in
its request body in binary format. The proof consists of a single byte indicating
the signing algorithm, a signature, the public key of the one making the request and an 8 Byte
unsigned integer representing the number of seconds since midnight on the 1st of January 1970 (UNIX timestamp)

Considering that currently only ED25519 is supported, a valid proof of authenticity
can be constructed as follows:

`algorithm byte (1 Byte) + signature (64 Bytes) + Public Key (32 Bytes) + UNIX timestamp (8 Bytes)`

The signature is created by signing the public key and the UNIX timestamp with the corresponding ED25519 private key

Since a timestamp is used to counteract replay attacks, the server and client are 
both requiered to have roughly the same system time. The use of NTP is therefore recommended. Currently the JAEM Message
Delivery Service rejects all proofs of authenticity older than 5 seconds. This
number has been arbitrarily chosen and may be subject to change in the future.

If the JAEM Message Delivery Service deems the proof valid, the response
will contain all messages send to the given public key since the previous request with a valid proof of authenticity. Each
Message is preceded by an 8 Byte unsigned integer (Big Endian) indicating the length of the following message.

Successfully requesting the stored messages, will stage them for deletion.
Each successfull request to this endpoint has to be followed by a request to the
`/delete_messages` endpoint to delete the messages from server storage.

## /delete_messages

The `/delete_messages` can be used to delete messages from server storage after they have 
been successfully retrieved. Each request to this endpoint therefore has to be preceded
by a successfull request to the `get_messages` endpoint or will otherwise not succeed.

Does no request to the `/delete_messages` enpoint follow within 20 seconds of a request to
the `get_messages` enpoint, the message will be unstaged from deletion.

A request to this
endpoint also has to contain a proof of authenticity in its request body in binary format.
The proof of authenticity has to follow the same structure as the proof for the `get_messages` endpoint
and has to be verified by the JAEM Message Delivery Service for the request to succeed.
