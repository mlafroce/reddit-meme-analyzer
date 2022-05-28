# Reddit meme analyzer

## DAG de entidades

![Grafo de entidades](dag.png)

En celeste las entidades trivialmente paralelizables utilizando particionado (por no poseer estado o porque el resultado  puede ser particionado y unificado con facilidad)

En rojo las entidades que no pueden ser fácilmente paralelizables. Por ejemplo, el calculador de mediana necesita otra entidad que tome todas las medianas y calcule una general.
El caso de Best Meme filter podría ser paralelizable, pero necesitaría otra entidad que ordene los resultados particionados.

El caso de `college comments filter` podría paralelizarse por palabras a buscar, pero generaría un overhead de mensajes considerable.

### Entidades

Cada entidad es una aplicación aparte, que se compila y ejecuta con el comando

```bash
cargo run --bin <nombre>
```

Las entidades presentes son

* `post_producer`: fuente de posts, genera `FullPost`s.
* `comment_producer`: fuente de comments, genera `FullComment`s.
* `url_extractor`: recibe `FullPost`s y los extrae Id y Url en un mensaje `PostUrl`.
* `score_extractor`: recibe `FullPost`s y extrae el Score en un mensaje `PostScore`.
* `mean_calculator`: recibe `FullScore`s y calcula la mediana generando un `PostScoreMean`.
* `best_meme_filter`: primero recibe los sentiments de los posts (mensajes PostSentiment), se queda con el mejor, y luego recibe los mensajes `PostUrl`, para obtener la url del post con más alto sentiment.
* `comment_sentiment_extractor`: recibe `FullComment`s y extrae el sentiment y postId asociado, generando un `PostIdSentiment`.
* `post_sentiment_calculator`: recibe `PostIdSentiment`s por comentario y calcula la mediana por postId un `PostIdSentiment`.
* `result_consumer`: recibe `FullScore`s y calcula la mediana generando un `PostScoreMean`.
