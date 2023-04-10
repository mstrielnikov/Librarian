import org.apache.spark.sql.SparkSession
import org.apache.spark.sql.functions._
import org.apache.tika.parser.pdf.PDFParser
import org.apache.tika.metadata.Metadata
import org.apache.tika.sax.BodyContentHandler
import org.elasticsearch.action.index.IndexRequest
import org.elasticsearch.common.xcontent.XContentType
import org.elasticsearch.client.{RestClient, RestHighLevelClient}
import org.elasticsearch.action.search.{SearchRequest}
import java.io.{FileInputStream, File}

// Define a UDF to extract the text from each PDF file
val ExtractTextUDF = udf((content: Array[Byte]) => {
  val handler = new BodyContentHandler()
  val metadata = new Metadata()
  val parser = new PDFParser()
  parser.parse(new FileInputStream(new File("/tmp")), handler, metadata, new ParseContext())
  handler.toString
})
 
val SearchSourceBuilderSizeLimit = 10000

class ElasticSearchETL(val sparkSession: SparkSession, val client: RestHighLevelClient, val indexName: String, val indexType: String) {
  
  def etlLoad(loadPath: String): Unit = {
    // Load the PDF files into a DataFrame
    val pdfFiles = sparkSession.read.binaryFiles(loadPath)
      .select("path", "content")
      .withColumn("filename", regexp_extract(col("path"), "[^/]*$", 0))

    // Extract the text from each PDF file using the UDF
    val pdfText = pdfFiles
      .withColumn("text", ExtractTextUDF(col("content")))
      .select("filename", "text")
      .as[PDFFile]

    // Index each PDF file in Elasticsearch
    pdfText.foreach(pdf => {
      val indexRequest = new IndexRequest(indexName, indexType, pdf.filename)
        .source(Map("text" -> pdf.text).asJava, XContentType.JSON)
      client.index(indexRequest)
    })
  }

  def getKeysByIndexValue(indexValue: String): Seq[String] = {
    val searchSourceBuilder = new SearchSourceBuilder()
      .query(QueryBuilders.termQuery("index_field_name", indexValue))
      .size(SearchSourceBuilderSizeLimit)

    val searchRequest = new SearchRequest(indexName)
      .source(searchSourceBuilder)

    val searchResponse = client.search(searchRequest)

    val hits = searchResponse.getHits.getHits

    val keys = ListBuffer[String]()

    hits.foreach(hit => keys += hit.getId)

    keys.toList
  }
}  
