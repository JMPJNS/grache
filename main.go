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
		Addr:     getEnv("REDIS_HOST", "localhost:6379"),
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

type GraphqlRequest struct {
	Query         string                 `json:"query"`
	OperationName string                 `json:"operationName"`
	Variables     map[string]interface{} `json:"variables"`
	Cookie        string
}

func handleRequest(w http.ResponseWriter, r *http.Request, rdb *redis.Client) {
	skipCache := false
	route := getEnv("URL", "http://localhost:3000/shop-api")
	// convert body into gql request
	bodyBytes, _ := io.ReadAll(r.Body)

	var gql GraphqlRequest
	err := json.Unmarshal(bodyBytes, &gql)
	if err != nil {
		fmt.Println(err)
	}
	query, _ := parser.ParseQuery(&ast.Source{Input: gql.Query})

	// set session cookie
	ignoreCookies, err := strconv.ParseBool(r.URL.Query().Get("ignoreCookies"))
	if err != nil {
		ignoreCookies = true
	}
	if ignoreCookies {
		r.Header.Del("Cookie")
	} else {
		sessionCookie, err := r.Cookie("session")
		if err == nil && sessionCookie != nil {
			gql.Cookie = sessionCookie.Value
		}
	}
	// get hash
	hashI, err := hashstructure.Hash(gql, hashstructure.FormatV2, nil)
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
		// TODO return proper headers aswell
		val, err := rdb.Get(ctx, hash).Result()
		if err != nil {
			fmt.Println(err)
		} else {
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
		fmt.Printf("client: error making http request: %s\n", err)
		os.Exit(1)
	}
	responseBytes, err := io.ReadAll(res.Body)
	responseString := string(responseBytes)

	expiration, err := strconv.Atoi(r.URL.Query().Get("expiration"))
	if err != nil {
		expiration = 10 * 60
	}
	err = rdb.Set(ctx, hash, responseString, time.Duration(expiration)*time.Second).Err()
	if err != nil {
		fmt.Println(err)
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
	fmt.Printf("%s \n", r.Method)
}

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}
