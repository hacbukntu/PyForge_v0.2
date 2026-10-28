importar "gui";
importar "integraciones";
importar "vpn";
importar "comandos";
importar "sistema";
importar "json";
importar "time";
// Actualiza un panel informativo; detectar VPN no activa ni verifica un túnel.
funcion actualizar(ventana: entero) -> nada {
    variable red_local = vpn.detectar();
    variable proceso = comandos.ejecutar("/usr/bin/printf", "[\"%s\",\"Proceso local correcto\"]", 2);
    gui.cambiar_texto(ventana, "estado", json.decodificar(json.obtener(proceso, "stdout")));
    gui.cambiar_texto(ventana, "red", red_local);
}
variable diseno = "{\"titulo\":\"Centro de operaciones ST\",\"tema\":\"oscuro\",\"controles\":[{\"tipo\":\"etiqueta\",\"id\":\"estado\",\"texto\":\"Preparando\"},{\"tipo\":\"etiqueta\",\"id\":\"red\",\"texto\":\"Estado local\"},{\"tipo\":\"boton\",\"id\":\"actualizar\",\"texto\":\"Actualizar\"},{\"tipo\":\"boton\",\"id\":\"cerrar\",\"texto\":\"Cerrar\"}]}";
variable ventana = gui.crear(diseno);
variable abierta = verdadero;
variable lista_gui = falso;
variable pruebas = sistema.entorno("ST_GUI_PRUEBA") == "1";
variable consultas = 0;
variable control = "";
intentar {
    mostrar(integraciones.informacion());
    mientras abierta {
        variable evento = gui.evento(ventana);
        variable nombre = json.decodificar(json.obtener(evento, "evento"));
        si nombre == "lista" { lista_gui = verdadero; actualizar(ventana); }
        si nombre == "click" {
            control = json.decodificar(json.obtener(evento, "id"));
            si control == "actualizar" { actualizar(ventana); }
            si control == "cerrar" { abierta = falso; }
        }
        si nombre == "error" { lanzar "La GUI informó un error; no se considera éxito"; }
        si nombre == "cerrada" { abierta = falso; }
        consultas = consultas + 1;
        si pruebas y consultas >= 50 { abierta = falso; }
        // dormir recibe milisegundos; 20 mantiene fluida la GUI y permite que la
        // prueba automática cierre en aproximadamente un segundo.
        variable times: entero = 10000;
        time.dormir(times);
    }
    si no lista_gui { lanzar "La ventana no llegó a estar preparada"; }
    mostrar("Panel abierto y cerrado correctamente");
} finalmente { gui.cerrar(ventana); }
