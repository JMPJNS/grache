package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"github.com/mitchellh/hashstructure/v2"
	"github.com/redis/go-redis/v9"
	"github.com/vektah/gqlparser/v2/ast"
	"github.com/vektah/gqlparser/v2/parser"
	"io"
	"log"
	"net/http"
	"os"
	"strconv"
	"time"
)

var ctx = context.Background()

func main() {
	redisDb, err := strconv.Atoi(getEnv("REDIS_DB", "0"))
	if err != nil {
		redisDb = 0
	}
	rdb := redis.NewClient(&redis.Options{
		Addr:     getEnv("REDIS_HOST", "127.0.0.1:6379"),
		Password: getEnv("REDIS_PASSWORD", ""),
		DB:       redisDb,
	})

	mux := http.NewServeMux()
	s := &http.Server{
		Addr:    fmt.Sprintf(":%s", getEnv("PORT", "3333")),
		Handler: mux,
	}
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) { handleRequest(w, r, rdb) })

	log.Fatal(s.ListenAndServe())
}

type RequestContext struct {
	Query         string                 `json:"query"`
	OperationName string                 `json:"operationName"`
	Variables     map[string]interface{} `json:"variables"`
	Cookie        string
}

func handleRequest(w http.ResponseWriter, r *http.Request, rdb *redis.Client) {
	if r.Method != http.MethodPost {
		w.WriteHeader(400)
		w.Write([]byte("Only POST Method supported"))
		return
	}
	skipCache := false
	route := getEnv("URL", "http://127.0.0.1:3000/shop-api")
	// convert body into requestContext
	bodyBytes, _ := io.ReadAll(r.Body)

	var requestContext RequestContext
	err := json.Unmarshal(bodyBytes, &requestContext)
	if err != nil {
		fmt.Println(err)
	}
	query, _ := parser.ParseQuery(&ast.Source{Input: requestContext.Query})

	// set session cookie
	ignoreCookies, err := strconv.ParseBool(r.URL.Query().Get("ignoreCookies"))
	if err != nil {
		ignoreCookies = false
	}
	if ignoreCookies {
		r.Header.Del("Cookie")
	} else {
		sessionCookie, err := r.Cookie("session")
		if err == nil && sessionCookie != nil {
			requestContext.Cookie = sessionCookie.Value
		}
	}
	// get hash
	hashI, err := hashstructure.Hash(requestContext, hashstructure.FormatV2, nil)
	hash := strconv.FormatUint(hashI, 10)

	// check if mutation
	// TODO somehow merge cached querries and mutations if both a query and a mutation are sent in a single request
	for _, value := range query.Operations {
		if value.Operation == "mutation" {
			// for now, we just always forward the entire request if it contains a mutation
			skipCache = true
		}
	}

	// get response from cache
	if !skipCache {
		val, err := rdb.Get(ctx, hash).Result()
		if err != nil {
			log.Println("Cache miss")
		} else {
			// TODO return proper headers aswell
			w.Write([]byte(val))
			return
		}
	}

	// fetch response if not found and save to redis
	// create the post request
	req, err := http.NewRequest(http.MethodPost, route, bytes.NewReader(bodyBytes))
	req.Header = r.Header
	client := http.Client{
		Timeout: 30 * time.Second,
	}
	res, err := client.Do(req)
	if err != nil {
		log.Printf("client: error making http request: %s\n\n", err)
		w.WriteHeader(500)
		w.Write([]byte(err.Error()))
		return
	}
	responseBytes, err := io.ReadAll(res.Body)
	responseString := string(responseBytes)

	expiration, err := strconv.Atoi(r.URL.Query().Get("expiration"))
	if err != nil {
		expiration = 10 * 60
	}
	err = rdb.Set(ctx, hash, responseString, time.Duration(expiration)*time.Second).Err()
	if err != nil {
		log.Println("Error writing entry to redis", err.Error())
	}

	// set response headers
	for key, value := range res.Header {
		for i := 0; i < len(value); i++ {
			w.Header().Add(key, value[i])
		}
	}

	// respond
	w.Write(responseBytes)
	return
}

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}
