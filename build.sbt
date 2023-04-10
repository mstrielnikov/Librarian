val version = "0.1.0"
val name = "fulltext-doc-search"
val description = "Text parser, indexer and searcher across PDF docs & other sources"

val scalaVersion = "3.2.2"
val sparkVersion = "3.3.2"

libraryDependencies ++= Seq(
    ("org.apache.spark" %% "spark-core" % sparkVersion).cross(CrossVersion.for3Use2_13),
    ("org.apache.spark" %% "spark-sql" % sparkVersion).cross(CrossVersion.for3Use2_13),
    "org.elasticsearch.client" % "elasticsearch-rest-high-level-client" % "7.17.9",
    "org.elasticsearch.client" % "elasticsearch-rest-client" % "8.7.0"
)