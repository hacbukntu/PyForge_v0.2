// Recursos nativos tipados con cierre automático por ámbito.
// `usar` adquiere la propiedad del resultado de una fábrica de recursos.
importar "recursos";

funcion mostrar_buffer(datos: buffer_nativo) -> nada {
    // El parámetro es un préstamo: esta función no puede cerrarlo ni retornarlo.
    mostrar(recursos.texto(datos));
}

usar datos = recursos.desde_texto("Hola desde ST") {
    mostrar(tipo(datos));
    mostrar_buffer(datos);
    recursos.poner(datos; 0; 104);
    mostrar(recursos.texto(datos));
} // El compilador genera aquí el cierre, incluso si el bloque lanza o retorna.

usar archivo = recursos.abrir_archivo("recursos-demo.txt"; 1) {
    usar contenido = recursos.desde_texto("Creado sin Python ni JSON") {
        mostrar(recursos.escribir_archivo(archivo; contenido));
    }
}
