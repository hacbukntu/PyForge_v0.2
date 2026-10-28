importar "db";
importar "json";
// Lee una fila de una consulta pequeña y comprueba el resultado antes de modificarlo.
funcion existencias(conexion: entero) -> entero {
    variable filas = db.consultar(conexion, "SELECT cantidad FROM inventario WHERE codigo=?", "[\"ST-01\"]");
    retornar json.a_entero(json.elemento(json.elemento(filas, 0), 0));
}
// El débito se valida dentro de la misma transacción; no permite inventario negativo.
funcion retirar(conexion: entero, cantidad: entero) -> nada {
    variable actual = existencias(conexion);
    si cantidad <= 0 o cantidad > actual { lanzar "Stock insuficiente o cantidad inválida"; }
    db.ejecutar(conexion, "UPDATE inventario SET cantidad=cantidad-? WHERE codigo=?", "[" + cantidad + ",\"ST-01\"]");
}
variable conexion = db.conectar("inventario.db");
intentar {
    db.ejecutar(conexion, "CREATE TABLE IF NOT EXISTS inventario(codigo TEXT PRIMARY KEY,cantidad INTEGER CHECK(cantidad>=0))", "[]");
    db.ejecutar(conexion, "INSERT OR IGNORE INTO inventario VALUES (?,?)", "[\"ST-01\",10]");
    db.confirmar(conexion);
    db.ejecutar(conexion, "BEGIN IMMEDIATE", "[]");
    retirar(conexion, 3);
    db.confirmar(conexion);
    mostrar("Existencias: " + existencias(conexion));
    intentar { retirar(conexion, 99); }
    capturar error { db.revertir(conexion); mostrar("Rechazo esperado: " + error); }
    db.respaldar(conexion, "inventario-respaldo.db");
} finalmente { db.cerrar(conexion); }
