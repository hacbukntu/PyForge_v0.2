importar "db";
variable conexion = db.conectar("agenda.db");
intentar {
    db.ejecutar(conexion, "CREATE TABLE IF NOT EXISTS personas(id INTEGER PRIMARY KEY,nombre TEXT)", "[]");
    // Parámetros separados: incluso las comillas son datos.
    db.ejecutar(conexion, "INSERT INTO personas(nombre) VALUES (?)", "[\"Ana O'Reilly\"]");
    db.confirmar(conexion);
    mostrar(db.consultar(conexion, "SELECT id,nombre FROM personas ORDER BY id LIMIT 10", "[]"));
} finalmente { db.cerrar(conexion); }
