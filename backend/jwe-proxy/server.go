package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
)

func server() {
	httpBind := os.Getenv("HTTP_BIND")
	if httpBind == "" {
		httpBind = ":2820"
	}

	done := make(chan os.Signal, 1)
	signal.Notify(done, os.Interrupt, syscall.SIGINT, syscall.SIGTERM)

	httpServer := &http.Server{
		Addr:    httpBind,
		Handler: proxy(nil),
	}

	go func() {
		fmt.Printf("🚀 JWE Proxy listening on %s\n", httpBind)
		err := httpServer.ListenAndServe()
		if err != nil && err != http.ErrServerClosed {
			fmt.Fprintf(os.Stderr, "HTTP Server error: %s\n", err)
			os.Exit(1)
		}
	}()

	<-done
	fmt.Println("🌙 JWE Proxy closing")
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	err := httpServer.Shutdown(ctx)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error while closing HTTP Server: %s\n", err)
	}
	fmt.Println("🏳‍🌈 Server closed")
}
