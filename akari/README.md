# akari

HTTP(S) 1.0 client.

## Behavior

If the server hit returns 200 OK, the reponse body is returned as bytes.

Suitable connections will be kept open and reused on subsequent requests.

## Issues and limitations

* Only the "GET" request method is supported.
* Since only response status code 200 is handled, redirections will trigger an error.
* A timeout mechanism for DNS resolution has not been implemented.
* IPv6 addresses are not supported.
* URL handling might not be sophisticated enough to cover all valid cases.
* Response size is limited to 50 Megabytes.
