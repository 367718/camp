# akari

Non-compliant HTTP(S) 1.1 client.

## Behavior

If the server hit returns 200 OK, the reponse body is returned as bytes.

Suitable connections will be kept open and reused on subsequent requests.

## Issues and limitations

* Only the "GET" request method is supported.
* Only responses with status code 200 are handled, so redirections will trigger an error.
* A timeout mechanism for DNS resolution has not been implemented.
* IPv6 addresses are not supported.
* URL handling might not be sophisticated enough to cover all valid cases.
* Response size is limited to 50 Megabytes.
