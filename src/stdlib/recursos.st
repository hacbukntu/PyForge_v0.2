// Recursos tipados con propiedad exclusiva de un bloque usar y préstamos durante llamadas.
// No existe cerrar público: el compilador genera el cierre al abandonar el bloque.
// La API nativo de enteros permanece disponible por compatibilidad, sin conversiones entre APIs.

// Adquiere un archivo nuevo: 0 lectura, 1 creación exclusiva, 2 truncado explícito.
funcion abrir_archivo(ruta: cadena; modo: entero) -> archivo_nativo {
    retornar aux_abrir(ruta; modo);
}
// Adquiere un buffer cero de longitud fija; el propietario debe ser un bloque usar.
funcion buffer(tamano: entero) -> buffer_nativo { retornar aux_buffer(tamano); }
// Copia los bytes de una cadena a un buffer nuevo y de propiedad independiente.
funcion desde_texto(texto: cadena) -> buffer_nativo { retornar aux_desde_texto(texto); }
// Presta el archivo durante la lectura y adquiere un buffer nuevo que requiere su propio usar.
funcion leer(archivo: archivo_nativo; cantidad: entero) -> buffer_nativo {
    retornar aux_leer(archivo; cantidad);
}
// Presta ambos recursos y devuelve el número de bytes escritos; no transfiere propiedad.
funcion escribir_archivo(archivo: archivo_nativo; datos: buffer_nativo) -> entero {
    retornar aux_escribir(archivo; datos);
}
// Consulta longitud en bytes de un préstamo vivo.
funcion tamano(datos: buffer_nativo) -> entero { retornar aux_tamano(datos); }
// Consulta un byte por índice no negativo.
funcion obtener(datos: buffer_nativo; indice: entero) -> entero { retornar aux_obtener(datos; indice); }
// Modifica un byte durante la llamada; no guarda el préstamo para después.
funcion poner(datos: buffer_nativo; indice: entero; valor: entero) -> nada {
    aux_poner(datos; indice; valor);
}
// Valida UTF-8 y devuelve una copia independiente como cadena ST.
funcion texto(datos: buffer_nativo) -> cadena { retornar aux_texto(datos); }
