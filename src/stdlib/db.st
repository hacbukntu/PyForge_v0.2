// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Abre SQLite; devuelve un identificador que debe cerrarse.
funcion conectar(ruta: cadena) -> entero {
    retornar json.a_entero(integraciones.pedir("db.abrir", "[" + json.codificar(ruta) + "]"));
}

// Ejecuta una sentencia con parámetros JSON separados; devuelve filas afectadas.
funcion ejecutar(conexion: entero, sentencia: cadena, parametros: cadena) -> entero {
    json.comprobar(parametros);
    retornar json.a_entero(integraciones.pedir("db.ejecutar", "[" + json.de_entero(conexion) + "," + json.codificar(sentencia) + "," + parametros + "]"));
}

// Devuelve hasta 100 filas JSON; usa LIMIT para paginar.
funcion consultar(conexion: entero, sentencia: cadena, parametros: cadena) -> cadena {
    json.comprobar(parametros);
    retornar integraciones.pedir("db.consultar", "[" + json.de_entero(conexion) + "," + json.codificar(sentencia) + "," + parametros + "]");
}

// Confirma la transacción SQLite pendiente.
funcion confirmar(conexion: entero) -> booleano {
    retornar json.a_booleano(integraciones.pedir("db.confirmar", "[" + json.de_entero(conexion) + "]"));
}

// Revierte la transacción SQLite pendiente.
funcion revertir(conexion: entero) -> booleano {
    retornar json.a_booleano(integraciones.pedir("db.revertir", "[" + json.de_entero(conexion) + "]"));
}

// Cierra y revierte cambios no confirmados.
funcion cerrar(conexion: entero) -> booleano {
    retornar json.a_booleano(integraciones.pedir("db.cerrar", "[" + json.de_entero(conexion) + "]"));
}
// Copia SQLite coherente sin sobrescribir: confirmar o revertir antes de llamar.
funcion respaldar(conexion: entero, destino: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("db.respaldar", "[" + json.de_entero(conexion) + "," + json.codificar(destino) + "]"));
}
