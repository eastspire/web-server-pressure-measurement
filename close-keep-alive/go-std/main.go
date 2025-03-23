package main

import (
	"fmt"
	"net/http"
)

func helloHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Connection", "close")
	fmt.Fprintf(w, "Hello")
}

func main() {
	http.HandleFunc("/", helloHandler)	
	if err := http.ListenAndServe(":60000", nil); err != nil {
		fmt.Println("Error starting server:", err)
	}
}
