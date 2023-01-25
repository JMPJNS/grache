# Grache
Graphql Caching service

This service proxies graphql queries and caches them for 
a specified amount of time (configurable on a per-request basis).

It only caches queries, not mutations and checks the session 
Cookie (optional, enable on per-request basis)
to respond with the correct data for the currently authenticated User.

# Run the Service
The easiest way to run is with `docker compose up`

# ENV variables
- PORT = set the port it should listen on (default 3333)
- REDIS_HOST = (default localhost:6379)
- REDIS_PASSWORD = (default is without password)
- REDIS_DB = (default 0)
- URL = the URL of the graphql server you want to cache for

# Query parameters
- ignoreCookies (true/false) = ignore cookies for this request (default true), 
set this to false if you want to include session cookie for authentication
- expiration (integer) = how many seconds to cache the response for (default 600)