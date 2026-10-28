// Adaptador del motor 2D nativo al protocolo JSON. Django conserva el estado en la sesión.
importar "juego";
importar "json";
funcion estado_juego(solicitud: cadena) -> cadena {
    variable accion = json.decodificar(json.obtener(solicitud; "accion"));
    variable cuerpo: lista = [208; 207; 206];
    variable comida = 212;
    variable direccion = 0;
    variable puntos = 0;
    variable fin = falso;
    variable estado = "{}";
    variable posiciones = "[]";
    variable cantidad = 0;
    variable solicitada = 0;
    variable siguiente_cuerpo: lista = [];
    si accion != "reiniciar" {
        estado = json.obtener(solicitud; "estado");
        posiciones = json.obtener(estado; "cuerpo");
        cantidad = json.a_entero(json.obtener(estado; "cantidad"));
        cuerpo = [];
        para i en rango(cantidad) { agregar(cuerpo; json.a_entero(json.elemento(posiciones; i))); }
        comida = json.a_entero(json.obtener(estado; "comida"));
        direccion = json.a_entero(json.obtener(estado; "direccion"));
        puntos = json.a_entero(json.obtener(estado; "puntos"));
        fin = json.a_booleano(json.obtener(estado; "fin"));
        solicitada = json.a_entero(json.obtener(solicitud; "direccion"));
        si solicitada >= 0 y solicitada <= 3 y solicitada != (direccion + 2) % 4 { direccion = solicitada; }
        si no fin {
            siguiente_cuerpo = juego.avanzar_serpiente(cuerpo; direccion; comida);
            si longitud(siguiente_cuerpo) == 0 { fin = verdadero; }
            sino {
                si longitud(siguiente_cuerpo) > longitud(cuerpo) {
                    puntos = puntos + 10;
                    comida = juego.comida_libre(siguiente_cuerpo; comida + 73);
                }
                cuerpo = siguiente_cuerpo;
                si longitud(cuerpo) >= 100 o comida < 0 { fin = verdadero; }
            }
        }
    }
    variable posiciones_json: lista[cadena] = [];
    para i en rango(longitud(cuerpo)) { agregar(posiciones_json; json.de_entero(cuerpo[i])); }
    retornar json.objeto(["cuerpo"; "cantidad"; "comida"; "direccion"; "puntos"; "fin"];
        [json.arreglo(posiciones_json); json.de_entero(longitud(cuerpo)); json.de_entero(comida);
         json.de_entero(direccion); json.de_entero(puntos); json.de_booleano(fin)]);
}
