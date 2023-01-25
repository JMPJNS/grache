package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/mitchellh/hashstructure/v2"
	"github.com/redis/go-redis/v9"
	"github.com/vektah/gqlparser/v2/ast"
	"github.com/vektah/gqlparser/v2/parser"
	"io"
	"net/http"
	"os"
	"time"
)

var ctx = context.Background()

func main() {
	rdb := redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "", // no password set
		DB:       0,  // use default DB
	})

	http.HandleFunc("/shop-api", getRoot)

	// expiration in seconds

	err := http.ListenAndServe(":3334", nil)

	if errors.Is(err, http.ErrServerClosed) {
		fmt.Printf("server closed\n")
	} else if err != nil {
		fmt.Printf("error starting server: %s\n", err)
		os.Exit(1)
	}

	handleQuery(rdb)
}

func handleQuery(rdb *redis.Client) {
	err := rdb.Set(ctx, "key", "amogus", 60).Err()
	if err != nil {
		panic(err)
	}

	val, err := rdb.Get(ctx, "key").Result()
	if err != nil {
		panic(err)
	}
	fmt.Println("key", val)

	val2, err := rdb.Get(ctx, "key2").Result()
	if err == redis.Nil {
		fmt.Println("key2 does not exist")
	} else if err != nil {
		panic(err)
	} else {
		fmt.Println("key2", val2)
	}
}

type GraphqlRequest struct {
	Query         string                 `json:"query"`
	OperationName string                 `json:"operationName"`
	Variables     map[string]interface{} `json:"variables"`
	Cookie        string
}

func getRoot(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		return
	}
	// convert body into gql request
	bodyBytes, _ := io.ReadAll(r.Body)
	bodyString := string(bodyBytes[:])

	query, _ := parser.ParseQuery(&ast.Source{Input: bodyString})
	println(query)

	var gql GraphqlRequest
	err := json.Unmarshal(bodyBytes, &gql)
	if err != nil {
		fmt.Println(err)
	}
	// set session cookie
	// TODO ignore session cookie with request parameter
	sessionCookie, err := r.Cookie("session")
	if err == nil && sessionCookie != nil {
		gql.Cookie = sessionCookie.Value
	}
	// get hash
	hash, err := hashstructure.Hash(gql, hashstructure.FormatV2, nil)
	fmt.Println(hash)

	// TODO check if mutation or query
	// TODO check if response is found in redis and return response immediately

	// fetch response if not found and save to redis
	// create the post request
	req, err := http.NewRequest(http.MethodPost, "http://localhost:3000/shop-api", bytes.NewReader(bodyBytes))
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
	//bodyString := string(bodyBytes)
	//fmt.Println(bodyString)

	// TODO save to redis

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
