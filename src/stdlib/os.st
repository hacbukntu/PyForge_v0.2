// Operaciones de archivos del proceso actual: no ejecutan órdenes de shell.
// Devuelve el directorio absoluto actual; las rutas se interpretan desde este lugar.
funcion directorio_actual() -> cadena { 
    retornar aux_directorio(); 
    }
// Distingue ausencia (ENOENT/ENOTDIR) de otros errores, como falta de permisos.
funcion existe(ruta: cadena) -> booleano {
    variable estado: entero = aux_existe(ruta);
    si estado == 0 { 
        retornar verdadero;
        }

    si estado == -2 o estado == -20 { 
        retornar falso; 
        }
    lanzar "os.existe: error del sistema " + estado;
    retornar falso;
}
// Crea un único directorio; un nombre ya existente es un error capturable.
funcion crear_directorio(ruta: cadena) -> nada {
    variable estado: entero = aux_crear(ruta);
    si estado < 0 { 
        lanzar "os.crear_directorio: error del sistema " + estado; 
        }
}
// Borra un archivo o enlace simbólico, nunca un árbol de directorios.
funcion borrar_archivo(ruta: cadena) -> nada {
    variable estado: entero = aux_borrar(ruta);
    si estado < 0 { 
        lanzar "os.borrar_archivo: error del sistema " + estado; 
        }
}
