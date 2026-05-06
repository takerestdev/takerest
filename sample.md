---
method: GET
url: "{{BASE_URL}}/api/users/{{USER_ID}}"
headers:
  Content-Type: "application/json"
  Authorization: "Bearer {{TOKEN}}"
params:
  - key: page
    value: "1"
    enabled: true
  - key: limit
    value: "20"
    enabled: true
auth:
  type: none
---
# Get Users

Optional description/notes in markdown body.

## Body

```json
{
  "email": "{{USER_EMAIL}}"
}
```
