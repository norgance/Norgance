package main

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"os"
)

type Request struct {
	Canard string `json:"canard"`
}

func proxy(privateKey interface{}) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Only POST on root path
		if r.URL.Path != "/" || r.Method != http.MethodPost {
			http.Error(w, http.StatusText(http.StatusNotFound), http.StatusNotFound)
			return
		}

		requestBytes, err := ioutil.ReadAll(r.Body)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Body read error: %s\n", err)
			http.Error(w, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
			return
		}
		requestString := string(requestBytes)

		requestStruct, err := decrypt(requestString, privateKey)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Decrypt error: %s\n", err)
			http.Error(w, http.StatusText(http.StatusBadRequest), http.StatusBadRequest)
			return
		}
		fmt.Println("oui", requestString, requestStruct.Canard)

		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.Header().Set("X-Content-Type-Options", "nosniff")
		w.WriteHeader(http.StatusOK)
		fmt.Fprintln(w, "Hello, World!")
	})
}
