// Ejecuta desde una carpeta temporal: crea nativo-demo.bin sin sobrescribirlo.
// Buffers y archivos son recursos separados; cada uno se cierra en finalmente.
importar "nativo";
variable datos = nativo.buffer(3);
variable archivo = 0;
variable lectura = 0;
intentar {
    nativo.poner(datos; 0; 0);   // Byte NUL: un buffer no es una cadena terminada en cero.
    nativo.poner(datos; 1; 255); // Binario que no es UTF-8 válido.
    nativo.poner(datos; 2; 65);  // A
    archivo = nativo.abrir_archivo("nativo-demo.bin"; 1); // Creación exclusiva.
    intentar { mostrar(nativo.escribir_archivo(archivo; datos)); }
    finalmente { nativo.cerrar(archivo); }
} finalmente { nativo.cerrar(datos); }
archivo = nativo.abrir_archivo("nativo-demo.bin"; 0); // Lectura.
intentar {
    lectura = nativo.leer(archivo; 65536);
    intentar {
        mostrar(nativo.tamano(lectura)); // 3
        mostrar(nativo.obtener(lectura; 1)); // 255
        intentar { mostrar(nativo.texto(lectura)); }
        capturar error { mostrar(error); } // Error recuperable, no reinterpretación de bytes.
    } finalmente { nativo.cerrar(lectura); }
} finalmente { nativo.cerrar(archivo); }
