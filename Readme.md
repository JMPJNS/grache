# Grache
Graphql Caching service

This service proxies graphql queries and caches them for 
a specified amount of time (configurable on a per-request basis).

By Default, it only caches queries, not mutations (optionally can be cached aswell) and checks the session 
Cookie (optional, enable on per-request basis)
to respond with the correct data for the currently authenticated User.

It forwards any headers you set (except the Cookie header) to the graphql server.

# Comparison
### Without Cache
![without-cache.png](images%2Fwithout-cache.png)
### With Cache
![with-cache.png](images%2Fwith-cache.png)

# Run the Service
The easiest way to run is with `docker compose up`

# ENV variables
- PORT = set the port it should listen on (default 3333)
- REDIS_URL = (default redis://127.0.0.1/)
- URL = the URL of the graphql server you want to cache for

# Query parameters
- ignoreAuth (bool) = ignore authentication for this request (default false), 
set this to true if you don't care about authentication cookies
  - this is overwritten by the Grache-Ignore-Auth header
(faster and more memory efficient since less cache entries)
- expiration (integer) = how many seconds to cache the response for (default 600)
  - this is overwritten by the Grache-Expiration header
- cacheMutations (bool) = if mutations should get cached (default false)
  - this is overwritten by the Grache-Cache-Mutations header
- url (string) = what underlying graphql endpoint to target (default from URL env variable)
  - this is overwritten by the GRACHE_URL header
