package main

import (
	"fmt"
	"net/http"
)

func helloHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Connection", "keep-alive")
	fmt.Fprintf(w, "Hello, World!")
}

func main() {
	http.HandleFunc("/", helloHandler)
	fmt.Println("Starting server at :60000...")
	if err := http.ListenAndServe(":60000", nil); err != nil {
		fmt.Println("Error starting server:", err)
	}
}