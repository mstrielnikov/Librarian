import sttp.client3._
import sttp.client3.circe._
import io.circe.generic.auto._
import com.typesafe.config.ConfigFactory

case class Document(id: String, content: String)

object ElasticsearchClient {

  private val config = ConfigFactory.load()
  private val baseUrl = config.getString("elasticsearch.url")
  private val indexName = config.getString("elasticsearch.index")
  private val backend = HttpURLConnectionBackend()

  def indexDocument(doc: Document): Unit = {
    val request = basicRequest
      .body(doc)
      .post(uri"$baseUrl/$indexName/_doc/${doc.id}")
      .response(asJson[Map[String, Any]])

    val response = request.send(backend)
    if (response.code != 201) {
      println(s"Failed to index document: ${response.body}")
    } else {
      println(s"Document indexed successfully: ${doc.id}")
    }
  }
}