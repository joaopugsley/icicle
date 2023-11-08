class com.clubpenguin.puffleavatar.PuffleAvatarController
{
   var _SHELL;
   var _ENGINE;
   var _puffleAvatarMap;
   var _LOGGER;
   var _roomMovieClip;
   function PuffleAvatarController(shellMC, engineMC)
   {
      this._SHELL = shellMC;
      this._ENGINE = engineMC
      this._puffleAvatarMap = new Object();
      this._SHELL.addListener(this._SHELL.REMOVE_PLAYER,com.clubpenguin.util.Delegate.create(this,this.onRemovePlayer));
      _LOGGER = net.cpavalanche.loggers.DefaultLoggerFactory.createLogger("PuffleAvatarController");
   }
   function attachPuffle(playerObject)
   {      
      if(playerObject.attachedPuffle != undefined && playerObject.attachedPuffle != null)
      {
         var puffleData = playerObject.attachedPuffle;
         var puffleModel = _SHELL.getPath("clothing_sprites") + (750 + puffleData.typeID) + ".swf";
         var puffleWorldObject;
         if(this._puffleAvatarMap[puffleData.id] == undefined || this._puffleAvatarMap[puffleData.id] == null)
         {
            puffleWorldObject = new com.clubpenguin.puffleavatar.PuffleAvatar(puffleData,this._SHELL, this._ENGINE);
            this._puffleAvatarMap[puffleData.id] = puffleWorldObject;
            flash.external.ExternalInterface.call("console.log", _puffleAvatarMap.length)
         }
         else
         {
            puffleWorldObject = this._puffleAvatarMap[puffleData.id];
         }
         puffleWorldObject.loadPuffleAsset(puffleModel,playerObject.player_id);
      }
   }
   function update()
   {
      for(var _loc2_ in this._puffleAvatarMap)
      {
         this._puffleAvatarMap[_loc2_].update();
      }
   }
   function detachPuffle(puffleId)
   {
      this.destroyPuffleAvatar(puffleId);
   }
   function getPuffleClip(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         return this._puffleAvatarMap[puffleId].getPuffleClip();
      }
      return null;
   }
   function getPuffleAvatar(puffleId)
   {
      return this._puffleAvatarMap[puffleId];
   }
   function displayPuffleWidget(puffleId, widgetType, puffleItemId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].displayPuffleStatsWidget(widgetType,puffleItemId);
      }
   }
   function hidePuffle(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].hide();
      }
   }
   function showPuffle(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].show();
      }
   }
   function hideHat(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].hideHat();
      }
   }
   function showHat(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].showHat();
      }
   }
   function loadHat(puffleId, puffleHatVO)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].loadPuffleHat(puffleHatVO);
      }
   }
   function unloadHat(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].unloadPuffleHat();
      }
   }
   function isPuffleVisible(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         return this._puffleAvatarMap[puffleId].isPuffleVisible();
      }
      return false;
   }
   function getPuffleCanMove(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         return this._puffleAvatarMap[puffleId].puffleCanMove;
      }
      return false;
   }
   function updatePuffleFrame(puffleId, frameNumber)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].updatePuffleFrame(frameNumber);
      }
   }
   function getPuffleFrame(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         return this._puffleAvatarMap[puffleId].getPuffleFrame();
      }
      return null;
   }
   function setFrameUpdateEnabled(puffleId, enable)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].setFrameUpdateEnabled(enable);
      }
   }
   function addPuffleEffect(puffleId, effect)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].loadPuffleEffect(effect);
      }
   }
   function onRemovePlayer(playerId)
   {
      var _loc3_ = this._SHELL.getPlayerObjectById(playerId);
      if(_loc3_.attachedPuffle != undefined && _loc3_.attachedPuffle != null)
      {
         this.destroyPuffleAvatar(_loc3_.attachedPuffle.id);
      }
   }
   function puffleAvatarExists(puffleId)
   {
      return this._puffleAvatarMap[puffleId] != undefined && this._puffleAvatarMap[puffleId] != null;
   }
   function destroyPuffleAvatar(puffleId)
   {
      if(this.puffleAvatarExists(puffleId))
      {
         this._puffleAvatarMap[puffleId].destroy();
         this._puffleAvatarMap[puffleId] = null;
      }
   }
}