# Lake Concu

## Procesos detectados

Se reconocen 4 tipos de procesos

* Procesos *Barco*: se encarga de transportar turistas entre ciudades. Los barcos pueden inspeccionarse a sí mismos (según el evento disparado por el supervisor, son prefectura o inspector)

* Procesos *Viajero*: se transportan entre ciudades utilizando barcos. Adicionalmente, hay *viajeros* (turistas) que pueden viajar a otras ciudades por cuenta propia (no pueden viajar a otra ciudad mientras están viajando).

#### Procesos no implementados

* Procesos *Inspector*: Puede inspeccionar barcos ubicados en ciudades, mientras estén en el estado *puerto*. Controla los pasajeros.

* Procesos *Prefectura*: Idem proceso *inspector*, pero controla el barco.

### Estados de los procesos

* Proceso *Supervisor*: el supervisor no tiene estado, se mantiene esperando entradas del usuario para accionar los eventos de inspección.

* Procesos *Barco*: Cada barco puede estar en los siguientes estados
Travel,
  LeavePassengers,
  PickPassengers,
  Disembark
    * *En viaje*: el barco está navegando, e intenta anclar en un puerto (o espera a que esté disponible)

    * *Levantando pasajeros*: abre un fifo para comunicarse con los pasajeros que están esperando en el puerto.

    * *Descargando pasajeros*: abre el fifo correspondiente a cada pasajero y le pregunta si llegó a su destino o no. Luego se desliga de los que se bajaron.

    * *En puerto*: el barco está asociado a una ciudad, puede descargar y cargar viajeros. Puede haber un sólo barco por ciudad.

* Procesos *Viajero*: Cada viajero puede estar en los siguientes estados

    * *En ciudad*: el viajero "pasea" por una ciudad está asociado a la misma. Puede aleatoriamente cambiar de ciudad mientras está en este estado.

    * *En barco*: el viajero está en un barco, por lo tanto está asociado al mismo. En este caso, si el barco recibe un evento de inspección, el viajero recibirá tal evento, pudiendo ser multado.

Los procesos de tipo *Barco* poseen como variable de estado la ciudad en donde se encuentran. No pueden cambiar la ciudad mientras están en estado "puerto".
Tampoco pueden cambiar de estado "puerto" a "ciudad" si la ciudad destino tiene un barco dentro.

Los procesos tipo *Viajero* también poseen una variable ciudad y una variable con el barco en el que se encuentran. No pueden pasar de estado "barco" a "ciudad" si el barco en el que se encuentran no está en estado ciudad.

Tampoco puede pasar de estado "ciudad" a "barco" si no hay barcos disponibles en la ciudad.

## Comunicación entre procesos

### Entre barcos

Cada barco sincroniza sus arribos a los puertos utilizando locks exclusivos.
Se crea un lock por cada puerto y cada vez que un barco está por arribar, se toma el lock correspondiente al puerto.
Al momento de dejar el puerto se libera el lock, permitiendo que otro barco llegue.

### Barco con personas

#### Al momento de abordar

Hay varias restricciones para abordar el barco:

Para limitar la cantidad de personas en un barco se puede usar un semáforo por ciudad. Cada persona que quiere viajar en barco se encola en un *turnstile* **¿Pasa algo si pierde el orden?**

Por cada espacio libre que tiene el barco, el barco envía un *signal* a un semáforo **¿Cuál?**

Como el barco tiene que saber qué pasajeros tiene encima, se utiliza un *FIFO* para que los pasajeros registren su *PID* y fecha del boleto.

#### Al momento de arribar

Propuesta 1: Una vez que los pasajeros suben al barco, se quedan esperando en un FIFO propio hasta que el barco les avise que llegó a otra ciudad.

Cuando llegan a la ciudad, el procesos *barco* les envía mediante el fifo de cada pasajero el id de la ciudad a la que llegan para decidir si bajar o no.

Propuesta 2: Utilizar otro método bloqueante.... ¿Cuál? y destrabar con una señal

#### Descenso de pasajeros

El barco realiza *nro_pasajeros* lecturas en el *FIFO* para saber cuántos quieren descender. Los que quieren descender escriben su *PID* y continúan su comportamiento normal, y los que se quedan, escriben 0 y se vuelven a bloquear con su propio *FIFO*. Por cada *PID* distinta de 0 el barco suma un espacio disponible y remueve al pasajero que descendió de su lista.

### Barco con supervisores

Se utilizarán *FIFO*s para que el supervisor detecte la presencia de un barco en el puerto. Cuando un barco llega, abre un *FIFO* correspondiente al puerto y encola su *PID*. Cuando un proceso supervisor llegar a un puerto, intenta desencolar del *FIFO* un *PID*. Si no llegó un barco, el supervisor se queda bloqueado esperando a que llegue.

Antes de partir, si ningún supervisor revisó el barco, este desencola su pid para que no lo revisen cuando sale del puerto.

**Este fifo ¿Debe estar protegido?**

### Supervisor con pasajeros

No hay comunicación directa entre supervisor y pasajeros.

Cuando el barco recibe una señal de inspección, el barco chequea los tickets de los pasajeros y notifica a los infractores mediante la misma *FIFO* que utilizan para saber si llegaron o no a destino (básicamente la notificación de infracción es igual a una notificación de llegar a una ciudad)


## Creación y destrucción de IPCs

Para sincronizar la creación y destrucción de IPCs "globales" se utilizará un lock con un contador dentro. El proceso que lo cree guardará un 1. 

Cuando un proceso de cualquier tipo inicia, si no lo puede crear, lo abre y le suma 1 al valor leído, y lo escribe actualizado.

Cuando un proceso finaliza, le resta 1 al valor leído, y lo actualiza. Si el contador da 0, además, elimina los IPCs abiertos "globales".

