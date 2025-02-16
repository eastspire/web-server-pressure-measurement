package main

import "github.com/gin-gonic/gin"

func main() {
    gin.SetMode(gin.ReleaseMode)
    r := gin.New()
    gin.DisableConsoleColor()
    r.GET("/", func(c *gin.Context) {
        c.Header("Connection", "close")
        c.String(200, "Hello")
    })
    r.Run(":60000")
}
