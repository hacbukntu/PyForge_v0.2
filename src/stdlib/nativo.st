// ABI nativa 1: archivos y buffers binarios implementados en runtime/native/lib.rs.
// Los enteros identifican recursos, no direcciones. Cierra cada recurso en finalmente.
// No utiliza Python ni JSON. No aplica los permisos del módulo integraciones.

// Convierte resultados negativos del runtime en excepciones ST recuperables.
funcion aux_comprobar(valor: entero; operacion: cadena) -> entero {
    si valor == -10001 { lanzar "nativo." + operacion + ": recurso cerrado, desconocido o de otro tipo"; }
    si valor == -10002 { lanzar "nativo." + operacion + ": limite de 32 recursos vivos"; }
    si valor == -10003 { lanzar "nativo." + operacion + ": tamano, indice, byte, ruta o modo fuera de rango"; }
    si valor == -10004 { lanzar "nativo." + operacion + ": el buffer no contiene UTF-8 valido"; }
    si valor < 0 { lanzar "nativo." + operacion + ": error del sistema Linux " + cadena(0 - valor); }
    retornar valor;
}
// Versión del contrato de llamadas enlazado en el ejecutable.
funcion version() -> entero { retornar aux_version(); }
// Abre lectura (0), crea sin sobrescribir (1) o trunca explícitamente (2).
funcion abrir_archivo(ruta: cadena; modo: entero) -> entero {
    retornar aux_comprobar(aux_abrir(ruta; modo); "abrir");
}
// Cierra archivo o libera buffer. El identificador y sus alias dejan de ser válidos.
funcion cerrar(recurso: entero) -> nada {
    aux_comprobar(aux_cerrar(recurso); "cerrar");
}
// Reserva hasta 65536 bytes inicializados a cero. Longitud fija durante su vida útil.
funcion buffer(tamano: entero) -> entero {
    retornar aux_comprobar(aux_buffer(tamano); "buffer");
}
// Copia los bytes UTF-8; la longitud de este buffer cuenta bytes, no caracteres.
funcion desde_texto(texto: cadena) -> entero {
    retornar aux_comprobar(aux_desde_texto(texto); "desde_texto");
}
// Lee hasta cantidad bytes. Puede devolver menos; longitud cero indica EOF.
funcion leer(archivo: entero; cantidad: entero) -> entero {
    retornar aux_comprobar(aux_leer(archivo; cantidad); "leer");
}
// Escribe todos los bytes y retorna su cantidad. Un fallo puede dejar datos parciales.
funcion escribir_archivo(archivo: entero; datos: entero) -> entero {
    retornar aux_comprobar(aux_escribir(archivo; datos); "escribir");
}
// Devuelve el número de bytes de un buffer vivo.
funcion tamano(datos: entero) -> entero {
    retornar aux_comprobar(aux_longitud(datos); "longitud");
}
// Lee un byte 0..255 con índice no negativo y límites comprobados.
funcion obtener(datos: entero; indice: entero) -> entero {
    retornar aux_comprobar(aux_obtener(datos; indice); "obtener");
}
// Cambia un byte sin alterar la longitud; rechaza valores fuera de 0..255.
funcion poner(datos: entero; indice: entero; valor: entero) -> nada {
    aux_comprobar(aux_poner(datos; indice; valor); "poner");
}
// Valida UTF-8 y crea una copia como cadena ST; el buffer sigue necesitando cerrar.
funcion texto(datos: entero) -> cadena {
    aux_comprobar(aux_utf8(datos); "texto");
    retornar aux_texto(datos);
}
