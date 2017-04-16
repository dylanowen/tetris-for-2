package com.tetrisfortwo

import akka.actor.ActorSystem
import akka.event.LoggingAdapter
import akka.http.scaladsl.Http
import akka.http.scaladsl.model.{HttpRequest, HttpResponse}
import akka.http.scaladsl.server.Directives._
import akka.stream.ActorMaterializer
import akka.stream.scaladsl.Flow

import scala.concurrent.ExecutionContext
import scala.io.StdIn

/**
  * TODO add description
  *
  * @author dylan.owen
  * @since Apr-2017
  */
object ServerApp {
  def main(args: Array[String]): Unit = {
    if (args.length < 1) {
      println("Expected path for static resources")
      sys.exit(1)
    }
    val staticRoot = args(0)
    val host: String = "0.0.0.0"
    val port: Int = if (args.length >= 2) args(1).toInt else 8080

    implicit val system: ActorSystem = ActorSystem("TetrisServer")
    implicit val materializer: ActorMaterializer = ActorMaterializer()
    implicit val log: LoggingAdapter = system.log
    // needed for the future flatMap/onComplete in the end
    implicit val executionContext: ExecutionContext = system.dispatcher

    val route: Flow[HttpRequest, HttpResponse, Any] = encodeResponse {
        getFromDirectory(staticRoot)
      }

    val bindingFuture = Http().bindAndHandle(route, host, port)

    println(s"Server online at http://" + host + ":" + port + "/\nPress RETURN to stop...")
    StdIn.readLine() // let it run until user presses return
    bindingFuture
      .flatMap(_.unbind()) // trigger unbinding from the port
      .onComplete(_ => system.terminate()) // and shutdown when done
  }
}
