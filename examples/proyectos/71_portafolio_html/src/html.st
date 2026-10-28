// Escapa datos antes de insertarlos en HTML o XML; el orden evita escapar dos veces.
funcion escapar(texto: cadena) -> cadena {
    variable seguro = reemplazar(texto,"&","&amp;");
    seguro = reemplazar(seguro,"<","&lt;");
    seguro = reemplazar(seguro,">","&gt;");
    seguro = reemplazar(seguro,"\"","&quot;");
    retornar seguro;
}
