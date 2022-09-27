export function download_building_yaml(data) {
    var blob = new Blob([data], {type: 'text/yaml'});
    var a = document.createElement("a");
    var url = URL.createObjectURL(blob);
    a.href = url;
    a.download = 'warehouse.building.yaml';
    a.click();
}
