// Adaptadores a las operaciones de texto existentes: mismos límites y validación UTF-8.
// Lee una línea con un mensaje opcionalmente vacío; EOF devuelve cadena vacía.
funcion leer_linea(mensaje: cadena) -> cadena { 
    retornar entrada(mensaje); 
    }
// Lee un archivo completo como texto UTF-8, hasta el límite nativo de 1 MiB.
funcion leer_texto(ruta: cadena) -> cadena { 
    retornar abrir(ruta); 
    }
// Crea o reemplaza el contenido de un archivo; no añade automáticamente un salto de línea.
funcion escribir_texto(ruta: cadena, contenido: cadena) -> nada { 
    escribir(ruta, contenido); 
    }
// Imprime una cadena con salto de línea; para otros tipos sigue disponible mostrar().
funcion imprimir(texto: cadena) -> nada { 
    mostrar(texto); 
    }
