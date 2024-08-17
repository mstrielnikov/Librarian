import org.apache.spark.sql.{DataFrame, SparkSession}
import org.apache.spark.sql.functions._
import java.nio.file.{Files, Paths}
import scala.util.Try

object TextUploader {

  def main(args: Array[String]): Unit = {
    val spark = SparkSession.builder
      .appName("TextUploader")
      .getOrCreate()

    // Load image paths from a file or directory (this example assumes a simple text file with image paths)
    val imagePaths = spark.read.textFile("path/to/image_paths.txt")

    // Define a DataFrame with image paths and extracted text
    val textData = imagePaths.rdd.map { path =>
      val text = OCRUtil.extractText(path)
      (path, text)
    }.toDF("id", "text")

    // Convert DataFrame to RDD and index each document
    textData.rdd.foreachPartition { partition =>
      partition.foreach { row =>
        val id = row.getAs[String]("id")
        val text = row.getAs[String]("text")
        val document = Document(id, text)
        ElasticsearchClient.indexDocument(document)
      }
    }

    spark.stop()
  }
}