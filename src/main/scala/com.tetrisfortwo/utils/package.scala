package com.tetrisfortwo

import akka.http.scaladsl.model.Uri.Path
import akka.http.scaladsl.model._
import akka.http.scaladsl.model.headers.RawHeader
import akka.http.scaladsl.server.Directives._
import akka.http.scaladsl.server.PathMatcher.{Matched, Unmatched}
import akka.http.scaladsl.server.{Directive1, PathMatcher0, Route}

import scala.collection.immutable

/**
  * TODO add description
  *
  * @author dylan.owen
  * @since Apr-2017
  */
package object utils {
  val UNAUTHORIZED: Route = buildError(
    401,
    "unauthorized",
    immutable.Seq(RawHeader("WWW-Authenticate", "TODO fill this in"))
  )
  val NOT_FOUND: Route = buildError(404, "not found")

  // create a path matcher that doesn't consume the slash
  val UnconsumedSlash: PathMatcher0 = new PathMatcher0 {
    def apply(path: Path) = path match {
      case Path.Slash(_) => Matched(path, ())
      case _ => Unmatched
    }
  }

  def buildError(statusCode: Int,
                 message: String,
                 headers: immutable.Seq[HttpHeader] = Nil): Route = {
    requireContentType(ContentTypes.`application/json`) {
      simpleComplete(statusCode, _, headers, s"""{"error": "$message"}""")
    } ~ simpleComplete(statusCode, ContentTypes.`text/html(UTF-8)`, headers,
      s"""
        |<html>
        | <title>$statusCode</title>
        | <body>$statusCode: $message</body>
        |</html>
      """.stripMargin
    )
  }

  private def requireContentType[T <: ContentType](contentType: T): Directive1[T] = {
    extractRequestEntity.flatMap(_.contentType match {
      case `contentType` => provide(contentType)
      case _ => reject
    })
  }

  def simpleComplete(statusCode: Int,
                     contentType: ContentType.NonBinary,
                     headers: immutable.Seq[HttpHeader],
                     body: String): Route = {
    encodeResponse
      complete(
        HttpResponse(
          statusCode,
          headers = headers,
          entity = HttpEntity(contentType, body)
        )
      )
  }
}
