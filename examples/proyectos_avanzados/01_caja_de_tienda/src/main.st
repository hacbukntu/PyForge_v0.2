importar "db";
importar "json";
importar "io";
// Guarda importes en centavos: evita redondeo binario para el dinero.
funcion cobrar(conexion: entero, unidades: entero) -> entero {
    si unidades < 1 o unidades > 100 { lanzar "Cantidad permitida: 1 a 100"; }
    variable total = unidades * 1250;
    db.ejecutar(conexion, "INSERT INTO ventas(unidades,total) VALUES (?,?)", "[" + unidades + "," + total + "]");
    retornar total;
}
variable conexion = db.conectar("caja.db");
intentar {
    db.ejecutar(conexion, "CREATE TABLE IF NOT EXISTS ventas(id INTEGER PRIMARY KEY,unidades INTEGER,total INTEGER)", "[]");
    variable unidades = entero(io.leer_linea("Unidades de cuaderno: "));
    variable total = cobrar(conexion, unidades);
    db.confirmar(conexion);
    mostrar("Total en centavos: " + total);
    mostrar(json.objeto(["unidades", "total_centavos"], [json.de_entero(unidades), json.de_entero(total)]));
} finalmente { db.cerrar(conexion); }
