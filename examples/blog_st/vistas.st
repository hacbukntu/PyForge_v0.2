// Vista nativa: transforma datos sin construir HTML a partir de entradas del visitante.
importar "modelos.st";
funcion resumen(titulo: cadena; autor: cadena) -> cadena {
    si (titulo_valido(titulo)) {
        retornar titulo + " — " + autor;
    } sino {
        retornar "Título inválido";
    }
}
