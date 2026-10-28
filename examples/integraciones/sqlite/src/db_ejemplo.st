// Ejemplo local de cuentas: SQL parametrizado y hashes, nunca contraseñas guardadas.
// Requiere permisos base_datos, leer_archivos, escribir_archivos y cifrado del proyecto.
importar "db";
importar "cifrado";
importar "json";

// Confirma la transacción únicamente si SQLite acepta el registro completo.
funcion registrar(correo: cadena; clave_plana: cadena) -> nada {
    variable conexion = db.conectar("usuarios.db");
    variable resumen = "";
    intentar {
        db.ejecutar(conexion; "CREATE TABLE IF NOT EXISTS usuarios(id INTEGER PRIMARY KEY, correo TEXT UNIQUE, clave_hash TEXT)"; "[]");
        resumen = cifrado.hash_clave(clave_plana);
        // json.codificar escapa comillas: no construyas parámetros JSON por concatenación manual.
        db.ejecutar(conexion; "INSERT INTO usuarios(correo, clave_hash) VALUES (?,?)";
            json.arreglo([json.codificar(correo); json.codificar(resumen)]));
        db.confirmar(conexion);
        mostrar("Usuario registrado");
    } capturar error {
        db.revertir(conexion);
        mostrar("Registro rechazado");
    } finalmente {
        db.cerrar(conexion);
    }
}

// db.consultar devuelve texto JSON, no una lista ST de filas: se decodifica explícitamente.
funcion iniciar_sesion(correo: cadena; clave_plana: cadena) -> booleano {
    variable conexion = db.conectar("usuarios.db");
    variable exito = falso;
    variable filas = "[]";
    variable resumen = "";
    intentar {
        filas = db.consultar(conexion; "SELECT clave_hash FROM usuarios WHERE correo = ? LIMIT 1";
            json.arreglo([json.codificar(correo)]));
        si filas != "[]" {
            resumen = json.decodificar(json.elemento(json.elemento(filas; 0); 0));
            exito = cifrado.verificar_clave(clave_plana; resumen);
        }
    } finalmente {
        db.cerrar(conexion);
    }
    retornar exito;
}

// Credenciales ficticias, solo para el ejemplo. No es el registro HTTP del foro.
registrar("ana@example.invalid"; "Clave-de-prueba-123!");
registrar("ana@example.invalid"; "Otra-clave-de-prueba!");
mostrar(iniciar_sesion("ana@example.invalid"; "Clave-de-prueba-123!"));
mostrar(iniciar_sesion("ana@example.invalid"; "incorrecta"));
mostrar(iniciar_sesion("ausente@example.invalid"; "incorrecta"));
