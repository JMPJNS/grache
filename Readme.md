# Grache
Graphql Caching service

This service proxies graphql querries and caches them for a specified ammount of time.
It only caches querries, not mutations and checks the session Cookie (option to disable on per request basis)
to respond with the correct data for the currently authenticated User.

# ENV variables
- PORT = set the port it should listen on (default 3333)
- REDIS_HOST = (default localhost:6379)
- REDIS_PASSWORD = (default is without password)
- REDIS_DB = (default 0)
- URL = the URL of the graphql server you want to cache for