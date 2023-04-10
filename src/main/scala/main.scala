import org.apache.spark.sql.SparkSession
import org.elasticsearch.client.{RestClient, RestHighLevelClient}
import scala.collection.JavaConverters._
import com.typesafe.config._

//Parse config file with static values
val Config = ConfigFactory.load()                                                  // Load properties from default location (classpath)
val ConfigLoadPathPDF = config.getString("FullTextDocSearch.load_file_path_PDF")   // Set the path to the directory containing PDF files
val ConfigEsPort = config.getString("FullTextDocSearch.elasticsearch_port")        // Set the ES port
val ConfigEsHost = config.getString("FullTextDocSearch.elasticsearch_host")        // Set the ES host
val ConfigEsIndexPDF = config.getString("FullTextDocSearch.index_pdf") 
val ConfigEsTypePDF = config.getString("FullTextDocSearch.index_pdf_type") 

object Main {
  def main(args: Array[String]): Unit = {
    // Create a Spark session
    val spark = SparkSession.builder
      .appName("FullTextDocSearch")
      .master("local[*]")
      .getOrCreate()

    // Create an Elasticsearch client
    val EsClient = new RestHighLevelClient(RestClient.builder(s"{ConfigEsHost}:{ConfigEsPort}"))

    ElasticSearchETL.etlLoad(EsClient, ConfigLoadPathPDF, ConfigEsIndexPDF, ConfigEsIndexTypePDF)
    
    docList := ElasticSearchETL.getKeysByIndexValue(args(0))

    docList.foreach(println)

    // Close the Elasticsearch client
    EsClient.close()

    // Stop the Spark session
    spark.stop()
  }
}